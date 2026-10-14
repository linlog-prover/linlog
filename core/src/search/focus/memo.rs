// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The memo of stable sequents. A proved sequent is a fact about its two
//! zones that holds regardless of how the search reached it; a failed one
//! is a fact only relative to the copy budget that was left when it failed,
//! unless the search space below it was explored without ever hitting the
//! budget. A complete failure holds as well for every sequent that differs
//! in interchangeable members of the linear zone only, so it is kept under
//! a key with every such member replaced by the first of its class (the
//! *canonical* key). A proof names its occurrences and stays under the
//! sequent's own key, and so does a failure cut by the budget: shared, it
//! would answer for a relative that the search of the sequent itself
//! leads to, level after level, and keep both from ever being complete.
//! One table holds all three: a key that is not canonical holds a proof
//! or a cut failure, a canonical one may also hold a complete failure,
//! which is all a relative reads there. The table has a cap in entries
//! and takes its memory from the search's account; when the search runs on
//! several threads it is one shard of a sharded map, [`Shared`](crate::search::focus::memo::Shared), whose
//! merge under the shard's lock keeps the same invariant: an entry's
//! validity only ever grows.
//!
//! A key is sparse: the unrestricted zone by the id an
//! [`Interner`](crate::search::focus::memo::Interner) gave
//! it, and the linear zone by its members. A stable sequent's linear zone
//! has a few members on a forest of tens of thousands of occurrences, and
//! its unrestricted zone, which only grows along a branch, is shared by
//! every stable sequent above the next `?` rule; with both zones as
//! bitsets of the forest's width, hashing and comparing the keys was up to
//! two fifths of a search.

use super::scratch::Pooled;
use crate::occurrences::{OccId, OccSet};
use crate::proofs::NodeId;
use crate::search::memory::{Account, bytes_of};
use std::hash::{BuildHasher as _, Hasher as _};
use std::sync::Mutex;

/// A stable sequent as the memo keys it, held: the unrestricted zone by
/// its id, the linear zone by its members.
#[derive(Debug, Default)]
pub(super) struct Key {
    /// The id of the unrestricted zone ([`Interner::intern`]).
    pub(super) theta: u64,
    /// The members of the linear zone in ascending order, an occurrence
    /// repeated as often as it is present.
    pub(super) members: Pooled<OccId>,
}

impl Key {
    /// Makes this key a copy of the zones, reusing the buffer, whose
    /// growth the caller charges ([`Pooled::settle`]).
    pub(super) fn assign(&mut self, zones: Zones<'_>) {
        self.theta = zones.theta;
        self.members.clear();
        self.members.extend_from_slice(zones.members);
    }

    /// The key's zones.
    pub(super) fn zones(&self) -> Zones<'_> {
        Zones {
            theta: self.theta,
            members: &self.members,
        }
    }
}

/// A stable sequent as the memo reads it: the id of the unrestricted
/// zone and the members of the linear zone, where they are. A lookup
/// copies nothing; an insertion copies the members into the memo's own
/// record.
#[derive(Clone, Copy, Debug)]
pub(super) struct Zones<'a> {
    /// The id of the unrestricted zone ([`Interner::intern`]).
    pub(super) theta: u64,
    /// The members of the linear zone in ascending order, an occurrence
    /// repeated as often as it is present.
    pub(super) members: &'a [OccId],
}

impl Zones<'_> {
    /// The hash of the sequent: the members folded two to a word by one
    /// multiplication each (rustc-hash's step, a bijection of the fold for
    /// each word), and the fold, the id and the count mixed by the crate's
    /// hasher. Hashing them one by one cost five times as much on a zone
    /// of a few dozen members, more than the bitsets of a narrow forest.
    pub(super) fn hash(self) -> u64 {
        /// rustc-hash's multiplier.
        const K: u64 = 0xf135_7aea_2e62_a9c5;
        let fold = |folded: u64, word: u64| (folded.rotate_left(26) ^ word).wrapping_mul(K);
        let (pairs, rest) = self.members.as_chunks::<2>();
        let mut folded = 0;
        for [first, second] in pairs {
            folded = fold(
                folded,
                u64::from(first.get()) | (u64::from(second.get()) << 32),
            );
        }
        if let [last] = rest {
            folded = fold(folded, u64::from(last.get()));
        }
        let mut hasher = crate::hash::BuildHasher::default().build_hasher();
        hasher.write_u64(self.theta);
        hasher.write_u64(folded);
        hasher.write_usize(self.members.len());
        hasher.finish()
    }

    /// Whether a key holds these zones.
    pub(super) fn of(self, key: &Key) -> bool {
        key.theta == self.theta && *key.members == *self.members
    }
}

/// The unrestricted zones a search has met, each under an id of its own,
/// by which a key names its zone in one word. Two zones have the same id
/// only if they are equal, since an id is never given twice: emptying the
/// table, when it is full or to give its memory back, only gives a zone
/// met again a new id, under which the memo finds nothing it recorded
/// under the old one. Equal zones on one branch still share their id:
/// the unrestricted zone only grows along a branch, so an equal one is the
/// same value, interned once. A search without a memo keeps no zone and
/// gives each a new id, which is all its loop check needs.
#[derive(Debug, Default)]
pub(super) struct Interner {
    /// The zones held, with their ids.
    ids: crate::hash::HashMap<OccSet, u64>,
    /// The id the next zone gets. One per zone interned, each a rule of
    /// the search that hashed a zone: no search reaches 2⁶⁴ of them.
    next: u64,
    /// The most zones held; zero keeps none.
    limit: usize,
    /// The bytes charged for the table and its zones.
    charged: usize,
}

impl Interner {
    /// Returns an empty table holding at most `limit` zones.
    pub(super) fn new(limit: usize) -> Self {
        Self {
            limit,
            ..Self::default()
        }
    }

    /// Returns the id of the zone: the one it has if the table holds it,
    /// else a new one, kept with the zone if the table has the room. A
    /// full table is emptied first; one the account cannot spare the
    /// memory for keeps nothing more.
    pub(super) fn intern(&mut self, zone: &OccSet, account: &Account) -> u64 {
        if self.limit != 0
            && let Some(&id) = self.ids.get(zone)
        {
            return id;
        }
        let id = self.next;
        self.next += 1;
        if self.limit == 0 {
            return id;
        }
        if self.ids.len() >= self.limit {
            self.release(account);
        }
        let set = size_of_val(zone.words());
        // A full table doubles its buckets: asked for before, as the
        // memo's index is.
        let capacity = self.ids.capacity();
        let buckets = if self.ids.len() < capacity {
            0
        } else {
            Self::buckets(capacity + 1) - Self::buckets(capacity)
        };
        if account.spares(set + buckets) {
            self.ids.insert(zone.clone(), id);
            let bytes = self.ids.len() * set + Self::buckets(self.ids.capacity());
            account.resize(self.charged, bytes);
            self.charged = bytes;
        }
        id
    }

    /// The bytes of the table's buckets where it holds `capacity` zones:
    /// a power of two of them, four for up to three zones, eight for up
    /// to seven, else eight for every seven, of a slot and a control byte
    /// each, and a group of control bytes after them; none for an empty
    /// table.
    fn buckets(capacity: usize) -> usize {
        /// The control bytes a table has after its buckets.
        const GROUP: usize = 16;
        let buckets = match capacity {
            0 => return 0,
            1..4 => 4,
            4..8 => 8,
            _ => (capacity * 8 / 7).next_power_of_two(),
        };
        buckets * (size_of::<(OccSet, u64)>() + 1) + GROUP
    }

    /// Empties the table and gives its memory back. The ids given stay
    /// given.
    pub(super) fn release(&mut self, account: &Account) {
        account.release(self.charged);
        self.charged = 0;
        self.ids = crate::hash::HashMap::default();
    }
}

/// What the search found out about a stable sequent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Entry {
    /// Provable, by the subproof rooted at this node of the engine's arena.
    Proved(NodeId),
    /// Unprovable, as far as the copy budget allowed.
    Failed(Failure),
}

/// How a stable sequent failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Failure {
    /// Every branch below it was explored to its end without hitting the
    /// copy budget: unprovable at any budget.
    Complete,
    /// Some branch below it was cut by the copy budget when this many
    /// copies were left: unprovable with at most that many copies left.
    Exhausted(u32),
    /// A complete failure, and with Mix one of every part of the linear
    /// zone as well: no non-empty sub-multiset of it is provable with the
    /// same unrestricted zone, at any budget.
    Hereditary,
}

impl Entry {
    /// The entry as one word of a record: the node of a proof, or a tag
    /// above the low half with the budget of a cut failure in it.
    fn code(self) -> u64 {
        match self {
            Self::Proved(node) => u64::from(node.get()),
            Self::Failed(Failure::Complete) => 1 << 32,
            Self::Failed(Failure::Exhausted(left)) => (2 << 32) | u64::from(left),
            Self::Failed(Failure::Hereditary) => 3 << 32,
        }
    }

    /// The entry a record's word stands for.
    fn of(code: u64) -> Self {
        match code >> 32 {
            0 => Self::Proved(NodeId::new(code as u32)),
            1 => Self::Failed(Failure::Complete),
            3 => Self::Failed(Failure::Hereditary),
            _ => Self::Failed(Failure::Exhausted(code as u32)),
        }
    }
}

/// What became of an insertion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Inserted {
    /// The entry is recorded, or the memo is switched off.
    Done,
    /// The table is full, by its entries or by the memory left: emptied,
    /// it takes the entry.
    Full,
    /// The table is empty and the memory left has no room for its first
    /// entry.
    NoRoom,
}

/// Where the key's hash is in a record, in two words, the low half first.
const HASH: usize = 0;

/// Where the entry is in a record, in two words.
const ENTRY: usize = 2;

/// Where the id of the unrestricted zone is in a record, in two words.
const THETA: usize = 4;

/// Where the number of the linear zone's members is in a record.
const MEMBERS: usize = 6;

/// The words of a record before the linear zone's members.
const HEADER: usize = 7;

/// The words of a chunk of an engine's own memo: a mebibyte.
const CHUNK: usize = 1 << 18;

/// The words of a chunk of one shard of a shared memo.
#[cfg(feature = "parallel")]
const SHARD_CHUNK: usize = 1 << 14;

/// Returns the number in two words of a record from `at` on.
fn read(words: &[u32], at: usize) -> u64 {
    u64::from(words[at]) | (u64::from(words[at + 1]) << 32)
}

/// Writes a number into two words of a record from `at` on.
fn write(words: &mut [u32], at: usize, value: u64) {
    words[at] = value as u32;
    words[at + 1] = (value >> 32) as u32;
}

/// Puts the record that starts at `at` into the first free slot from its
/// hash on.
fn place(slots: &mut [u64], at: u64, hash: u64) {
    let mask = slots.len() - 1;
    let mut slot = hash as usize & mask;
    while slots[slot] != 0 {
        slot = (slot + 1) & mask;
    }
    slots[slot] = at + 1;
}

/// The memo: stable sequents mapped to what the search found out about
/// them, with at most `limit` entries, within the memory the search's
/// account has left. When the table is full it is emptied, which only
/// costs time: every entry is a fact the search can find again.
///
/// An entry is a record of 32-bit words: its key's hash, the entry, the
/// id of the unrestricted zone, the number of the linear zone's members
/// and the members. Records lie one after another in chunks of a fixed
/// size (a record longer than a chunk has a chunk to itself), so the
/// memo's memory is the chunks it has asked for, an entry costs no
/// allocation of its own, emptying the table is resetting a count and
/// zeroing its index, and dropping it frees a few hundred blocks: a full
/// memo of a gibibyte went in half a second when every key was two
/// allocations. The index is open addressing over the places where
/// records start, which a search only ever looks up or adds to.
#[derive(Debug)]
pub(super) struct Memo {
    /// Per slot, where a record starts plus one (the chunk in the high
    /// half, the word in the low), or zero: linear probing from the hash's
    /// low bits. A power of two in length, at least twice the records, or
    /// empty.
    slots: Vec<u64>,
    /// The chunks, each allocated whole and filled record by record:
    /// records are only ever added, in order, and a record is read only
    /// once it is written, so a chunk is never zeroed first. A record's
    /// words are at most `u32::MAX`, so a chunk is too, and a word's
    /// place in its chunk fits the low half of a slot.
    chunks: Vec<Vec<u32>>,
    /// The chunk records are added to; those after it are empty.
    current: usize,
    /// The words a chunk has, known from the first record: zero until
    /// then.
    chunk: usize,
    /// The words a chunk is to have at most.
    most: usize,
    /// The number of records. Below `u32::MAX`, which bounds the index.
    len: usize,
    /// The number of entries the table holds at most; zero switches the memo
    /// off.
    limit: usize,
    /// The most entries the table held at once.
    peak: usize,
    /// How many lookups found an entry that applied.
    hits: u64,
}

impl Memo {
    /// Returns an empty memo holding at most `limit` entries.
    pub(super) fn new(limit: usize) -> Self {
        Self::with_chunk(limit, CHUNK)
    }

    /// Returns an empty memo holding at most `limit` entries in chunks of
    /// about `most` words.
    fn with_chunk(limit: usize, most: usize) -> Self {
        Self {
            slots: Vec::new(),
            chunks: Vec::new(),
            current: 0,
            chunk: 0,
            most,
            len: 0,
            limit: limit.min(u32::MAX as usize - 1),
            peak: 0,
            hits: 0,
        }
    }

    /// The words of the record that starts at `at`.
    fn record(&self, at: u64) -> &[u32] {
        let (chunk, start) = (&self.chunks[(at >> 32) as usize], at as u32 as usize);
        &chunk[start..start + HEADER + chunk[start + MEMBERS] as usize]
    }

    /// The words of the record that starts at `at`, to write.
    fn record_mut(&mut self, at: u64) -> &mut [u32] {
        let (chunk, start) = (&mut self.chunks[(at >> 32) as usize], at as u32 as usize);
        let end = start + HEADER + chunk[start + MEMBERS] as usize;
        &mut chunk[start..end]
    }

    /// Whether a record is the key's.
    fn matches(record: &[u32], key: Zones<'_>) -> bool {
        let members = &record[HEADER..];
        read(record, THETA) == key.theta
            && members.len() == key.members.len()
            && members.iter().zip(key.members).all(|(&m, o)| m == o.get())
    }

    /// Where the record of a key with the hash given starts, if there is
    /// one.
    fn find(&self, key: Zones<'_>, hash: u64) -> Option<u64> {
        if self.slots.is_empty() {
            return None;
        }
        let mask = self.slots.len() - 1;
        let mut slot = hash as usize & mask;
        loop {
            let at = self.slots[slot].checked_sub(1)?;
            let record = self.record(at);
            if read(record, HASH) == hash && Self::matches(record, key) {
                return Some(at);
            }
            slot = (slot + 1) & mask;
        }
    }

    /// The entry of the record that starts at `at`.
    fn entry(&self, at: u64) -> Entry {
        Entry::of(read(self.record(at), ENTRY))
    }

    /// [`insert_hashed`](Self::insert_hashed) for a key.
    #[cfg(test)]
    fn insert(&mut self, key: &Key, entry: Entry, account: &Account) -> Inserted {
        self.insert_hashed(key.zones(), key.zones().hash(), entry, account)
    }

    /// [`get_hashed`](Self::get_hashed) for a key.
    #[cfg(test)]
    fn get(&mut self, key: &Key, remaining: u32) -> Option<Entry> {
        self.get_hashed(key.zones(), key.zones().hash(), remaining)
    }

    /// Returns what is known about a stable sequent, whose hash is given,
    /// that applies with `remaining` copies left: a proof or a complete
    /// failure always, a failure cut by the budget only when at most as
    /// many copies are left now as were then, since a larger budget could
    /// prove more.
    fn get_hashed(&mut self, key: Zones<'_>, hash: u64, remaining: u32) -> Option<Entry> {
        let entry = self.entry(self.find(key, hash)?);
        if let Entry::Failed(Failure::Exhausted(then)) = entry
            && remaining > then
        {
            return None;
        }
        self.hits += 1;
        Some(entry)
    }

    /// [`refuted_hashed`](Self::refuted_hashed) for a key.
    #[cfg(test)]
    fn refuted(&mut self, key: &Key) -> bool {
        self.refuted_hashed(key.zones(), key.zones().hash())
            .is_some()
    }

    /// Returns the complete failure recorded under a canonical key, whose
    /// hash is given, if there is one: the answer for every sequent the
    /// key stands for. Whatever else the key holds is about the canonical
    /// sequent alone.
    fn refuted_hashed(&mut self, key: Zones<'_>, hash: u64) -> Option<Failure> {
        let refuted = self.find(key, hash).and_then(|at| match self.entry(at) {
            Entry::Failed(failure @ (Failure::Complete | Failure::Hereditary)) => Some(failure),
            _ => None,
        });
        self.hits += u64::from(refuted.is_some());
        refuted
    }

    /// Records what the search found out about a stable sequent, unless
    /// the table has no room for a new key: then it says so and the caller
    /// empties it ([`clear`](Self::clear)) and inserts again. A proof or a
    /// complete failure replaces anything; a failure cut by the budget
    /// only raises the budget an earlier such failure recorded. The memory
    /// a new chunk or a larger index takes is charged to `account`, and
    /// only taken while it leaves an eighth of the bound to what the
    /// search cannot empty ([`Account::spares`]). The key's hash is given.
    /// A key of more members than a record can count is never recorded,
    /// as if the memory were short.
    fn insert_hashed(
        &mut self,
        key: Zones<'_>,
        hash: u64,
        entry: Entry,
        account: &Account,
    ) -> Inserted {
        if self.limit == 0 {
            return Inserted::Done;
        }
        if let Some(at) = self.find(key, hash) {
            let old = self.entry(at);
            debug_assert!(
                !matches!(
                    (old, entry),
                    (
                        Entry::Proved(_),
                        Entry::Failed(Failure::Complete | Failure::Hereditary)
                    )
                ),
                "a complete failure of a proved sequent"
            );
            let keep = match (old, entry) {
                (
                    Entry::Failed(Failure::Exhausted(then)),
                    Entry::Failed(Failure::Exhausted(now)),
                ) => now <= then,
                (Entry::Proved(_) | Entry::Failed(Failure::Hereditary), Entry::Failed(_)) => true,
                (
                    Entry::Failed(Failure::Complete),
                    Entry::Failed(Failure::Complete | Failure::Exhausted(_)),
                ) => true,
                _ => false,
            };
            if !keep {
                write(self.record_mut(at), ENTRY, entry.code());
            }
            return Inserted::Done;
        }
        let words = HEADER + key.members.len();
        let counted = u32::try_from(words).is_ok();
        if self.len >= self.limit || !counted || !self.reserve(words, account) {
            return if self.len == 0 {
                Inserted::NoRoom
            } else {
                Inserted::Full
            };
        }
        let chunk = &mut self.chunks[self.current];
        let start = chunk.len();
        debug_assert!(chunk.capacity() - start >= words, "a chunk with room");
        chunk.resize(start + HEADER, 0);
        write(chunk, start + HASH, hash);
        write(chunk, start + ENTRY, entry.code());
        write(chunk, start + THETA, key.theta);
        chunk[start + MEMBERS] = key.members.len() as u32;
        chunk.extend(key.members.iter().map(|o| o.get()));
        self.len += 1;
        place(
            &mut self.slots,
            ((self.current as u64) << 32) | start as u64,
            hash,
        );
        self.peak = self.peak.max(self.len);
        Inserted::Done
    }

    /// Makes room for one more record of `words` words: an index of twice
    /// the size when half its slots are taken, and a chunk with the room,
    /// the current one or the next, made or grown where it has not, each
    /// only if the account has the memory. Returns whether there is room.
    fn reserve(&mut self, words: usize, account: &Account) -> bool {
        if self.chunk == 0 {
            // Under a small bound the chunks are small: a sixteenth of
            // the room there is, so that the table can grow in steps.
            let room = account.limit().saturating_sub(account.used()) / 16;
            self.chunk = (room / size_of::<u32>() as u64).clamp(1, self.most as u64) as usize;
        }
        if (self.len + 1) * 2 > self.slots.len() {
            let wider = (self.slots.len() * 2).max(16);
            if !account.spares(wider * size_of::<u64>()) {
                return false;
            }
            let narrower = bytes_of(&self.slots);
            self.slots = vec![0; wider];
            account.resize(narrower, bytes_of(&self.slots));
            for (c, chunk) in self.chunks.iter().enumerate().take(self.current + 1) {
                let mut start = 0;
                while start < chunk.len() {
                    let at = ((c as u64) << 32) | start as u64;
                    place(&mut self.slots, at, read(chunk, start + HASH));
                    start += HEADER + chunk[start + MEMBERS] as usize;
                }
            }
        }
        let room = |chunk: &Vec<u32>| chunk.capacity() - chunk.len();
        if self
            .chunks
            .get(self.current)
            .is_some_and(|chunk| !chunk.is_empty() && room(chunk) < words)
        {
            self.current += 1;
        }
        match self.chunks.get_mut(self.current) {
            Some(chunk) if room(chunk) >= words => true,
            Some(chunk) => {
                // An empty chunk that a record longer than a chunk grows.
                let (before, after) = (bytes_of(chunk), words * size_of::<u32>());
                if !account.spares(after - before) {
                    return false;
                }
                chunk.reserve_exact(words);
                account.resize(before, bytes_of(chunk));
                true
            }
            None => {
                let capacity = self.chunk.max(words);
                if !account.spares(capacity * size_of::<u32>()) {
                    return false;
                }
                let chunk = Vec::with_capacity(capacity);
                account.charge(bytes_of(&chunk));
                self.chunks.push(chunk);
                true
            }
        }
    }

    /// Empties the table and keeps its memory for the entries to come: a
    /// count is reset, the index zeroed and the chunks' lengths reset.
    pub(super) fn clear(&mut self) {
        self.len = 0;
        self.current = 0;
        self.slots.fill(0);
        for chunk in &mut self.chunks {
            chunk.clear();
        }
    }

    /// Empties the table and gives its memory back.
    pub(super) fn release(&mut self, account: &Account) {
        let held = bytes_of(&self.slots) + self.chunks.iter().map(bytes_of).sum::<usize>();
        account.release(held);
        self.len = 0;
        self.current = 0;
        self.slots = Vec::new();
        self.chunks = Vec::new();
    }

    /// Returns the most entries the table held at once.
    pub(super) fn peak(&self) -> usize {
        self.peak
    }

    /// Returns how many lookups found an entry that applied.
    pub(super) fn hits(&self) -> u64 {
        self.hits
    }
}

/// How many shards a shared memo has: a key's top hash bits pick one.
const SHARDS: usize = 64;

/// The memo shared by the workers of a parallel search: [`SHARDS`] memos
/// behind one lock each, a key's top hash bits choosing the shard, and
/// one table of the unrestricted zones behind a lock of its own, so that
/// every worker names a zone by the same id. A lookup or an insertion
/// holds one lock for the time of one map operation, so the merge of
/// `insert` is atomic per key and two workers that decide the same
/// sequent at once cost duplicated work, never a weaker entry. The cap is
/// per shard, and a shard that is full, by its entries or by the memory
/// left, is emptied by itself.
#[derive(Debug)]
pub(super) struct Shared {
    /// The shards.
    shards: Box<[Mutex<Memo>]>,
    /// The unrestricted zones the workers met.
    zones: Mutex<Interner>,
}

impl Shared {
    #[cfg(feature = "parallel")]
    /// Returns an empty shared memo holding at most `limit` entries in all.
    pub(super) fn new(limit: usize) -> Self {
        let per_shard = limit.div_ceil(SHARDS);
        Self {
            shards: (0..SHARDS)
                .map(|_| {
                    Mutex::new(Memo::with_chunk(
                        if limit == 0 { 0 } else { per_shard },
                        SHARD_CHUNK,
                    ))
                })
                .collect(),
            zones: Mutex::new(Interner::new(limit)),
        }
    }

    /// The shard of a key with the hash given.
    fn shard(&self, hash: u64) -> std::sync::MutexGuard<'_, Memo> {
        crate::search::lock(&self.shards[(hash >> (64 - SHARDS.trailing_zeros())) as usize])
    }

    /// [`Memo::get_hashed`] on the key's shard.
    pub(super) fn get(&self, key: Zones<'_>, hash: u64, remaining: u32) -> Option<Entry> {
        self.shard(hash).get_hashed(key, hash, remaining)
    }

    /// [`Memo::refuted_hashed`] on the key's shard.
    pub(super) fn refuted(&self, key: Zones<'_>, hash: u64) -> Option<Failure> {
        self.shard(hash).refuted_hashed(key, hash)
    }

    /// [`Memo::insert_hashed`] on the key's shard, which is emptied first
    /// when it is full. An entry that even the empty shard has no memory
    /// for is dropped: the other shards hold it, and the search says so
    /// when it finds itself over its bound.
    pub(super) fn insert(&self, key: Zones<'_>, hash: u64, entry: Entry, account: &Account) {
        let mut shard = self.shard(hash);
        if shard.insert_hashed(key, hash, entry, account) == Inserted::Full {
            shard.clear();
            shard.insert_hashed(key, hash, entry, account);
        }
    }

    /// Empties every shard and the table of zones and gives their memory
    /// back.
    pub(super) fn release(&self, account: &Account) {
        for shard in &self.shards {
            crate::search::lock(shard).release(account);
        }
        crate::search::lock(&self.zones).release(account);
    }

    /// Returns the sum over the shards of the most entries each held at
    /// once: an upper bound on the most entries the memo held at once.
    pub(super) fn peak(&self) -> usize {
        self.shards
            .iter()
            .map(|s| crate::search::lock(s).peak())
            .sum()
    }

    /// Returns how many lookups found an entry that applied.
    pub(super) fn hits(&self) -> u64 {
        self.shards
            .iter()
            .map(|s| crate::search::lock(s).hits())
            .sum()
    }
}

/// The memo an engine consults: its own, or the one a parallel search
/// shares among its workers.
#[derive(Debug)]
pub(super) enum Table<'a> {
    /// The engine's own memo, and its own table of unrestricted zones.
    Own(Memo, Interner),
    /// The memo of a parallel search.
    #[cfg_attr(
        not(feature = "parallel"),
        expect(dead_code, reason = "only a pool shares its memo")
    )]
    Shared(&'a Shared),
}

impl Table<'_> {
    /// An engine's own memo of at most `limit` entries, which holds as
    /// many unrestricted zones.
    pub(super) fn own(limit: usize) -> Self {
        Self::Own(Memo::new(limit), Interner::new(limit))
    }

    /// [`Interner::intern`].
    pub(super) fn intern(&mut self, zone: &OccSet, account: &Account) -> u64 {
        match self {
            Self::Own(_, zones) => zones.intern(zone, account),
            Self::Shared(shared) => crate::search::lock(&shared.zones).intern(zone, account),
        }
    }

    /// [`Memo::get_hashed`].
    pub(super) fn get(&mut self, key: Zones<'_>, hash: u64, remaining: u32) -> Option<Entry> {
        match self {
            Self::Own(memo, _) => memo.get_hashed(key, hash, remaining),
            Self::Shared(shared) => shared.get(key, hash, remaining),
        }
    }

    /// [`Memo::refuted_hashed`].
    pub(super) fn refuted(&mut self, key: Zones<'_>, hash: u64) -> Option<Failure> {
        match self {
            Self::Own(memo, _) => memo.refuted_hashed(key, hash),
            Self::Shared(shared) => shared.refuted(key, hash),
        }
    }

    /// [`Memo::insert_hashed`]. A shared memo makes its own room and always
    /// answers [`Inserted::Done`].
    pub(super) fn insert(
        &mut self,
        key: Zones<'_>,
        hash: u64,
        entry: Entry,
        account: &Account,
    ) -> Inserted {
        match self {
            Self::Own(memo, _) => memo.insert_hashed(key, hash, entry, account),
            Self::Shared(shared) => {
                shared.insert(key, hash, entry, account);
                Inserted::Done
            }
        }
    }

    /// [`Memo::clear`] on an engine's own memo; a shared memo empties its
    /// shards itself. The table of zones stays: emptied, it would give a
    /// zone met again a new id, under which neither the failures that
    /// wait nor the entries recorded after the clear are found.
    pub(super) fn clear(&mut self) {
        if let Self::Own(memo, _) = self {
            memo.clear();
        }
    }

    /// Empties the memo and the table of zones and gives their memory
    /// back.
    pub(super) fn release(&mut self, account: &Account) {
        match self {
            Self::Own(memo, zones) => {
                memo.release(account);
                zones.release(account);
            }
            Self::Shared(shared) => shared.release(account),
        }
    }

    /// [`Memo::peak`].
    pub(super) fn peak(&self) -> usize {
        match self {
            Self::Own(memo, _) => memo.peak(),
            Self::Shared(shared) => shared.peak(),
        }
    }

    /// Counts a visit that something beside the table answered as a hit
    /// of the table, in the shard of the key's hash.
    pub(super) fn count_hit(&mut self, hash: u64) {
        match self {
            Self::Own(memo, _) => memo.hits += 1,
            Self::Shared(shared) => shared.shard(hash).hits += 1,
        }
    }

    /// [`Memo::hits`].
    pub(super) fn hits(&self) -> u64 {
        match self {
            Self::Own(memo, _) => memo.hits(),
            Self::Shared(shared) => shared.hits(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A key with the unrestricted zone of id `theta` and the members
    /// given.
    fn key(theta: u64, members: &[u32]) -> Key {
        let mut key = Key {
            theta,
            ..Key::default()
        };
        key.members.extend(members.iter().map(|&o| OccId::new(o)));
        key
    }

    /// Under a canonical key only a complete failure answers for the
    /// sequents the key stands for: a cut failure and a proof there are
    /// the canonical sequent's own.
    #[test]
    fn complete_failures_are_shared() {
        let key = key(0, &[1]);
        let account = Account::new(None);
        let mut memo = Memo::new(10);
        assert!(!memo.refuted(&key));
        memo.insert(&key, Entry::Failed(Failure::Exhausted(1)), &account);
        assert!(!memo.refuted(&key));
        memo.insert(&key, Entry::Failed(Failure::Complete), &account);
        assert!(memo.refuted(&key));
        let mut proved = Memo::new(10);
        proved.insert(&key, Entry::Proved(NodeId::new(4)), &account);
        assert!(!proved.refuted(&key));
        assert_eq!((memo.hits(), proved.hits()), (1, 0));
    }

    /// A failure cut by the budget is a hit only with at most as many
    /// copies left, a complete failure and a proof always; a later entry
    /// only strengthens what is known.
    #[test]
    fn bounded_failures() {
        let account = Account::new(None);
        let mut memo = Memo::new(10);
        let one = key(0, &[1]);
        assert_eq!(memo.get(&one, 0), None);
        memo.insert(&one, Entry::Failed(Failure::Exhausted(1)), &account);
        assert_eq!(memo.get(&one, 2), None, "more copies left now");
        assert_eq!(
            memo.get(&one, 1),
            Some(Entry::Failed(Failure::Exhausted(1)))
        );
        assert_eq!(
            memo.get(&one, 0),
            Some(Entry::Failed(Failure::Exhausted(1)))
        );
        memo.insert(&one, Entry::Failed(Failure::Exhausted(0)), &account);
        assert_eq!(
            memo.get(&one, 1),
            Some(Entry::Failed(Failure::Exhausted(1))),
            "a smaller budget does not weaken the entry"
        );
        memo.insert(&one, Entry::Failed(Failure::Exhausted(3)), &account);
        assert_eq!(
            memo.get(&one, 3),
            Some(Entry::Failed(Failure::Exhausted(3)))
        );
        memo.insert(&one, Entry::Failed(Failure::Complete), &account);
        assert_eq!(memo.get(&one, 100), Some(Entry::Failed(Failure::Complete)));
        memo.insert(&one, Entry::Failed(Failure::Exhausted(5)), &account);
        assert_eq!(
            memo.get(&one, 100),
            Some(Entry::Failed(Failure::Complete)),
            "a complete failure stays"
        );
        let other = key(0, &[2]);
        let proved = Entry::Proved(NodeId::new(7));
        memo.insert(&other, proved, &account);
        assert_eq!(memo.get(&other, 0), Some(proved));
        memo.insert(&other, Entry::Failed(Failure::Exhausted(9)), &account);
        assert_eq!(memo.get(&other, 0), Some(proved), "a proof stays");
        assert_eq!(memo.hits(), 8);
        assert_eq!(memo.peak(), 2);
    }

    /// A table full by its entries says so and takes the entry once it is
    /// emptied, keeping its memory; one whose next chunk the account has
    /// no room for is full in the same way, and an empty one that cannot
    /// have its first chunk has no room; a release gives everything back.
    #[test]
    fn full_and_emptied() {
        let entry = Entry::Failed(Failure::Complete);
        let account = Account::new(None);
        let mut memo = Memo::new(2);
        assert_eq!(memo.insert(&key(0, &[1]), entry, &account), Inserted::Done);
        assert_eq!(memo.insert(&key(0, &[2]), entry, &account), Inserted::Done);
        assert_eq!(memo.insert(&key(0, &[1]), entry, &account), Inserted::Done);
        assert_eq!(memo.insert(&key(0, &[3]), entry, &account), Inserted::Full);
        let held = account.used();
        memo.clear();
        assert_eq!(memo.insert(&key(0, &[3]), entry, &account), Inserted::Done);
        assert!(memo.refuted(&key(0, &[3])) && !memo.refuted(&key(0, &[1])));
        assert_eq!(account.used(), held, "the emptied table keeps its memory");
        memo.release(&account);
        assert_eq!(account.used(), 0);
        assert!(!memo.refuted(&key(0, &[3])));

        // Under a bound the table grows in chunks of a sixteenth of the
        // room, here 64 words, eight records of eight, until no more fit:
        // here its index is what no longer does.
        let account = Account::new(Some(4096));
        let mut memo = Memo::new(1000);
        let fitted = (0..1000)
            .take_while(|&member| {
                memo.insert(&key(0, &[member]), entry, &account) == Inserted::Done
            })
            .count();
        assert_eq!(fitted, 64, "entries in 4 KiB");
        assert_eq!(memo.chunks.len(), 8);
        assert!(!account.over() && !account.spares(2 * bytes_of(&memo.slots)));
        assert_eq!(
            memo.insert(&key(0, &[1000]), entry, &account),
            Inserted::Full
        );
        memo.clear();
        assert_eq!(
            memo.insert(&key(0, &[1000]), entry, &account),
            Inserted::Done
        );
        assert_eq!(memo.peak(), fitted);
        let mut starved = Memo::new(100);
        assert_eq!(
            starved.insert(&key(0, &[0]), entry, &Account::new(Some(100))),
            Inserted::NoRoom
        );
    }

    /// Records of every length lie in the chunks one after another, a
    /// record longer than a chunk in one of its own, and each is found
    /// again, also after the index grew and after the table was emptied
    /// and filled anew; keys that differ in the unrestricted zone, a
    /// member or a copy of one are other keys.
    #[test]
    fn records() {
        let account = Account::new(None);
        let mut memo = Memo::with_chunk(1000, 64);
        let long: Vec<u32> = (0..100).collect();
        let keys: Vec<Key> = (0..3)
            .flat_map(|round| {
                let long = &long;
                (0..60u32).map(move |n| match n {
                    0 => key(round, long),
                    1 => key(round, &[7, 7]),
                    2 => key(round, &[]),
                    _ => key(round, &(0..=n % 9).map(|m| m * 3 + n).collect::<Vec<_>>()),
                })
            })
            .collect();
        for _ in 0..2 {
            for (i, key) in keys.iter().enumerate() {
                memo.insert(key, Entry::Failed(Failure::Exhausted(i as u32)), &account);
            }
            for (i, key) in keys.iter().enumerate() {
                assert_eq!(
                    memo.get(key, 0),
                    Some(Entry::Failed(Failure::Exhausted(i as u32)))
                );
            }
            assert_eq!(memo.get(&key(0, &[7]), 0), None);
            assert_eq!(memo.get(&key(3, &[7, 7]), 0), None);
            assert_eq!(memo.get(&key(0, &long[1..]), 0), None);
            memo.clear();
            assert_eq!(memo.get(&keys[1], 0), None);
        }
        assert!(memo.chunks.iter().any(|chunk| chunk.capacity() > 64));
    }

    /// Equal zones get one id, other zones others; ids stay unique when
    /// the table is emptied, when it is full and when it keeps nothing.
    #[test]
    fn interned_zones() {
        let account = Account::new(None);
        let zone = |members: &[u32]| OccSet::of(200, members.iter().map(|&o| OccId::new(o)));
        let mut zones = Interner::new(2);
        let a = zones.intern(&zone(&[1, 150]), &account);
        let b = zones.intern(&zone(&[1]), &account);
        assert_ne!(a, b);
        assert_eq!(zones.intern(&zone(&[1, 150]), &account), a);
        assert!(account.used() > 0);
        let c = zones.intern(&zone(&[2]), &account);
        assert!(![a, b].contains(&c), "a full table empties itself");
        assert_ne!(zones.intern(&zone(&[1]), &account), b);
        zones.release(&account);
        assert_eq!(account.used(), 0);
        assert!(zones.intern(&zone(&[1, 150]), &account) > c);
        let mut none = Interner::new(0);
        let first = none.intern(&zone(&[1]), &account);
        assert_ne!(none.intern(&zone(&[1]), &account), first);
        // Under a bound the table asks for its buckets before it doubles
        // them, as the memo does, and leaves the eighth of the bound free.
        let bounded = Account::new(Some(4096));
        let mut zones = Interner::new(1000);
        let kept = (0..200)
            .filter(|&o| {
                zones.intern(&zone(&[o]), &bounded);
                assert!(bounded.spares(0), "{} bytes held", bounded.used());
                zones.ids.contains_key(&zone(&[o]))
            })
            .count();
        assert!((8..100).contains(&kept), "{kept} zones kept");
    }
}
