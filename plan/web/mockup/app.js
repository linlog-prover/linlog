// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

// linlog, mockup 2: Instrument. A scripted prototype: every state comes from
// data/ (bundled into data.js); no library runs here.
(function () {
'use strict';

const D = window.LINLOG_DATA;
const $ = (s, r) => (r || document).querySelector(s);
const $$ = (s, r) => Array.from((r || document).querySelectorAll(s));
const SCALE = 0.03; // px per SVG unit at zoom 1: 1000 units = 30 px
const reducedMQ = matchMedia('(prefers-reduced-motion: reduce)');
const phoneMQ = matchMedia('(max-width: 760px)');
const DEBUG = new URLSearchParams(location.search).has('debug');
if (DEBUG) { addEventListener('error', e => { document.body.dataset.err = e.message; }); addEventListener('unhandledrejection', e => { document.body.dataset.err = String(e.reason && e.reason.stack || e.reason).replace(/\s+/g, " ").slice(0, 300); }); }
const esc = s => String(s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

// ---------- formulas as linlog draws them (Euler Math, ~X as X⊥, italic atoms)
function italic(ch) {
  const c = ch.charCodeAt(0);
  if (c >= 65 && c <= 90) return String.fromCodePoint(0x1D434 + c - 65);
  if (c >= 97 && c <= 122) return ch === 'h' ? 'ℎ' : String.fromCodePoint(0x1D44E + c - 97);
  return ch;
}
function mathHTML(s) {
  let out = '';
  for (let i = 0; i < s.length; i++) {
    const ch = s[i];
    if (ch === '~' && /[A-Za-z]/.test(s[i + 1] || '')) { out += italic(s[i + 1]) + '<sup>⊥</sup>'; i++; }
    else if (/[A-Za-z]/.test(ch)) out += italic(ch);
    else out += esc(ch);
  }
  return '<span class="m">' + out + '</span>';
}
const ruleHTML = r => '<span class="m">' + esc(r) + '</span>';
const speak = s => s.replace(/~/g, 'not ').replace(/⊢/g, 'turnstile');

// ---------- goals as the transcript prints them
function parseGoal(line) {
  const m = line.match(/^goal (\d+): (.*)$/);
  const n = +m[1], rest = m[2];
  const two = !rest.startsWith('⊢');
  const strip = side => side ? side.split(', ').map(x => x.replace(/^\d+: /, '')) : [];
  let left = [], right;
  if (two) { const [l, r] = rest.split(' ⊢ '); left = strip(l); right = strip(r); }
  else right = strip(rest.slice(2));
  const fs = left.concat(right);
  const seq = two ? left.join(', ') + ' ⊢ ' + right.join(', ') : '⊢ ' + right.join(', ');
  return { n, fs, seq, two, nLeft: left.length };
}

// ---------- the scripted proofs
const AX0 = 'the axiom needs formula 0 together with its dual literal and nothing else';
const axRefusal = p => 'the axiom needs formula ' + p + ' together with its dual literal and nothing else';
const P = {};

P.mll = {
  title: 'A ⊗ B ⊢ B ⊗ A', input: 'A * B |- B * A', uni: 'A ⊗ B ⊢ B ⊗ A', mode: 'classical', frag: 'MLL', engine: 'focus', dir: 'mll',
  net: 'mll/net-ids.svg', netOk: true,
  states: {
    s0: { svg: 'mll/interact/state-0.svg', json: 'mll/interact/state-0.json', goals: ['goal 0: ⊢ 0: ~A ⅋ ~B, 1: B ⊗ A'] },
    s1: { svg: 'mll/interact/state-1.svg', json: 'mll/interact/state-1.json', goals: ['goal 1: ⊢ 0: ~A, 1: ~B, 2: B ⊗ A'] },
    s2: { svg: 'mll/interact/state-2.svg', json: 'mll/interact/state-2.json', goals: ['goal 2: ⊢ 0: ~B, 1: B', 'goal 3: ⊢ 0: ~A, 1: A'] },
    s3: { svg: 'mll/interact/state-3.svg', json: 'mll/interact/state-3.json', goals: ['goal 3: ⊢ 0: ~A, 1: A'] },
    s4: { svg: 'mll/interact/state-4.svg', json: 'mll/interact/state-4.json', goals: [], done: true },
    d1: { svg: 'mll/dead-end/derived-state-split.svg', goals: ['goal 2: ⊢ 0: ~A, 1: B', 'goal 3: ⊢ 0: ~B, 1: A'] },
    c1: { svg: 'mll/interact/state-4.svg', json: 'mll/interact/state-4.json', goals: [], done: true, search: [1, 2, 3] },
    c0: { svg: 'mll/derivation-ids.svg', goals: [], done: true, search: [1, 2, 3, 4] },
    c2: { svg: 'mll/interact/state-4.svg', json: 'mll/interact/state-4.json', goals: [], done: true, search: [1, 2] },
  },
  moves: {
    's0 0 0': { '⅋': 's1' },
    's1 1 2': { split: { cands: [0, 1], res: { '1': { to: 's2', pass: true }, '0': { to: 'd1', pass: false }, '': { pass: false }, '0,1': { pass: false } } } },
    's1 1 0': { ax: { refuse: AX0 } }, 's1 1 1': { ax: { refuse: axRefusal(1) } },
    's2 2 0': { ax: 's3' }, 's2 2 1': { ax: 's3' },
    's3 3 0': { ax: 's4' }, 's3 3 1': { ax: 's4' },
    'd1 2 0': { ax: { refuse: AX0 } }, 'd1 2 1': { ax: { refuse: axRefusal(1) } },
    'd1 3 0': { ax: { refuse: AX0 } }, 'd1 3 1': { ax: { refuse: axRefusal(1) } },
  },
  close: {
    's0 0': { to: 'c0' }, 's1 1': { to: 'c1' },
    'd1 2': { verdict: 'unbalanced', text: '~A occurs 1 more time than A in the one-sided sequent, so they cannot all meet in axioms', atom: 'A', extra: '~A', seq: '⊢ ~A, B' },
  },
  closeAll: { s0: 'c0', s1: 'c1', s2: 'c2' },
  exports: {
    svg: 'mll/derivation.svg', png: 'mll/derivation.png', pdf: 'mll/derivation.pdf', tex: 'mll/derivation.tex', typ: 'mll/derivation.typ',
    json: 'mll/interact/proof.json', v: 'mll/interact/proof.v', txt: 'mll/prove.txt', netsvg: 'mll/net.svg', netpng: 'mll/net.png',
  },
};

P.first = {
  title: 'A, A ⊸ B ⊢ B', input: 'A, A -o B |- B', uni: 'A, A ⊸ B ⊢ B', mode: 'classical', frag: 'MLL', engine: 'focus', dir: 'first',
  net: 'first/net-ids.svg', netOk: true, ill: 'illfirst',
  states: {
    s0: { svg: 'first/interact/state-0.svg', json: 'first/interact/state-0.json', goals: ['goal 0: ⊢ 0: ~A, 1: A ⊗ ~B, 2: B'] },
    s1: { svg: 'first/interact/state-1.svg', json: 'first/interact/state-1.json', goals: ['goal 1: ⊢ 0: ~A, 1: A', 'goal 2: ⊢ 0: ~B, 1: B'] },
    s2: { svg: 'first/interact/state-2.svg', json: 'first/interact/state-2.json', goals: ['goal 2: ⊢ 0: ~B, 1: B'] },
    s3: { svg: 'first/interact/state-3.svg', json: 'first/interact/state-3.json', goals: [], done: true },
  },
  moves: {
    's0 0 1': { split: { cands: [0, 2], res: { '0': { to: 's1', pass: true }, '': { pass: false }, '2': { pass: false }, '0,2': { pass: false } } } },
    's0 0 0': { ax: { refuse: AX0 } }, 's0 0 2': { ax: { refuse: axRefusal(2) } },
    's1 1 0': { ax: 's2' }, 's1 1 1': { ax: 's2' },
    's2 2 0': { ax: 's3' }, 's2 2 1': { ax: 's3' },
  },
  close: {}, closeAll: {},
  exports: { svg: 'first/derivation.svg', tex: 'first/derivation.tex', typ: 'first/derivation.typ', json: 'first/interact/proof.json', v: 'first/interact/proof.v', txt: 'first/prove.txt', netsvg: 'first/net.svg' },
};

P.illfirst = {
  title: 'A, A ⊸ B ⊢ B', input: 'A, A -o B |- B', uni: 'A, A ⊸ B ⊢ B', mode: 'intuitionistic', frag: 'IMLL', engine: 'two-sided', dir: 'ill-first',
  net: null, netWhy: 'The essential net of an intuitionistic MLL proof appears here once the proof is finished.',
  states: {
    s0: { svg: 'ill-first/interact/state-0.svg', json: 'ill-first/interact/state-0.json', goals: ['goal 0: 0: A, 1: A ⊸ B ⊢ 2: B'] },
    s1: { svg: 'ill-first/interact/state-1.svg', json: 'ill-first/interact/state-1.json', goals: ['goal 1: 0: A ⊢ 1: A', 'goal 2: 0: B ⊢ 1: B'] },
    s2: { svg: 'ill-first/interact/state-2.svg', json: 'ill-first/interact/state-2.json', goals: ['goal 2: 0: B ⊢ 1: B'] },
    s3: { svg: 'ill-first/interact/state-3.svg', json: 'ill-first/interact/state-3.json', goals: [], done: true },
  },
  moves: {
    's0 0 1': { split: { cands: [0, 2], res: { '0': { to: 's1', pass: true }, '2': { refuse: 'a premise would have 2 formulas on the right of ⊢ instead of one' }, '0,2': { refuse: 'a premise would have 2 formulas on the right of ⊢ instead of one' }, '': { pass: false } } } },
    's1 1 0': { ax: 's2' }, 's1 1 1': { ax: 's2' }, 's2 2 0': { ax: 's3' }, 's2 2 1': { ax: 's3' },
  },
  close: {}, closeAll: {},
  exports: { svg: 'ill-first/derivation.svg', tex: 'ill-first/derivation.tex', typ: 'ill-first/derivation.typ', json: 'ill-first/interact/proof.json', v: 'ill-first/interact/proof.v', txt: 'ill-first/prove.txt' },
};

P.exp = {
  title: '!A ⊢ A ⊗ !A', input: '!A |- A * !A', uni: '!A ⊢ A ⊗ !A', mode: 'classical', frag: 'MELL', engine: 'focus', dir: 'exp',
  net: null, netWhy: 'No net view for this sequent: proof nets are drawn for MLL (and as essential nets for intuitionistic MLL), and this one has exponentials, ! and ?.',
  states: {
    s0: { svg: 'exp/interact/state-0.svg', json: 'exp/interact/state-0.json', goals: ['goal 0: ⊢ 0: ?~A, 1: A ⊗ !A'] },
    s1: { svg: 'exp/interact/state-1.svg', json: 'exp/interact/state-1.json', goals: ['goal 1: ⊢ 0: ?~A, 1: ?~A, 2: A ⊗ !A'] },
    s2: { svg: 'exp/interact/state-2.svg', json: 'exp/interact/state-2.json', goals: ['goal 2: ⊢ 0: ?~A, 1: A', 'goal 3: ⊢ 0: ?~A, 1: !A'] },
    s3: { svg: 'exp/interact/state-3.svg', json: 'exp/interact/state-3.json', goals: ['goal 3: ⊢ 0: ?~A, 1: !A', 'goal 4: ⊢ 0: ~A, 1: A'] },
    s4: { svg: 'exp/interact/state-4.svg', json: 'exp/interact/state-4.json', goals: ['goal 3: ⊢ 0: ?~A, 1: !A'] },
    s5: { svg: 'exp/interact/state-5.svg', json: 'exp/interact/state-5.json', goals: ['goal 5: ⊢ 0: ?~A, 1: A'] },
    s6: { svg: 'exp/interact/state-6.svg', json: 'exp/interact/state-6.json', goals: ['goal 6: ⊢ 0: ~A, 1: A'] },
    s7: { svg: 'exp/interact/state-7.svg', json: 'exp/interact/state-7.json', goals: [], done: true },
  },
  moves: {
    's0 0 0': { '?c': 's1' },
    's1 1 2': { split: { cands: [0, 1], res: { '0': { to: 's2', pass: true }, '1': { to: 's2', pass: true } } } },
    's2 2 0': { '?d': 's3' }, 's2 2 1': { ax: { refuse: axRefusal(1) } },
    's3 4 0': { ax: 's4' }, 's3 4 1': { ax: 's4' },
    's4 3 1': { '!': 's5' },
    's5 5 0': { '?d': 's6' }, 's5 5 1': { ax: { refuse: axRefusal(1) } },
    's6 6 0': { ax: 's7' }, 's6 6 1': { ax: 's7' },
  },
  close: {}, closeAll: {},
  exports: { svg: 'exp/derivation.svg', tex: 'exp/derivation.tex', typ: 'exp/derivation.typ', json: 'exp/interact/proof.json', v: 'exp/interact/proof.v', txt: 'exp/prove.txt' },
};

P.ill = {
  title: '1, A & B, B ⊸ C ⊢ C', input: '1, A & B, B -o C |- C', uni: '1, A & B, B ⊸ C ⊢ C', mode: 'intuitionistic', frag: 'IMALL', engine: 'two-sided', dir: 'ill',
  net: null, netWhy: 'No net view for this sequent: essential nets exist for intuitionistic MLL, and this one has the additive &.',
  states: {
    s0: { svg: 'ill/interact/state-0.svg', json: 'ill/interact/state-0.json', goals: ['goal 0: 0: 1, 1: A & B, 2: B ⊸ C ⊢ 3: C'] },
    s1: { svg: 'ill/interact/state-1.svg', json: 'ill/interact/state-1.json', goals: ['goal 1: 0: A & B, 1: B ⊸ C ⊢ 2: C'] },
    s2: { svg: 'ill/interact/state-2.svg', json: 'ill/interact/state-2.json', goals: ['goal 2: 0: B, 1: B ⊸ C ⊢ 2: C'] },
    s3: { svg: 'ill/interact/state-3.svg', json: 'ill/interact/state-3.json', goals: ['goal 3: 0: B ⊢ 1: B', 'goal 4: 0: C ⊢ 1: C'] },
    s4: { svg: 'ill/interact/state-4.svg', json: 'ill/interact/state-4.json', goals: ['goal 4: 0: C ⊢ 1: C'] },
    s5: { svg: 'ill/interact/state-5.svg', json: 'ill/interact/state-5.json', goals: [], done: true },
  },
  moves: {
    's0 0 0': { '1L': 's1' },
    's0 0 1': { '&L₁': { refuseNote: true } },
    's1 1 0': { '&L₂': 's2' },
    's2 2 1': { split: { cands: [0, 2], res: { '0': { to: 's3', pass: true }, '2': { refuse: 'a premise would have 2 formulas on the right of ⊢ instead of one' }, '0,2': { refuse: 'a premise would have 2 formulas on the right of ⊢ instead of one' }, '': { pass: false } } } },
    's2 2 0': { ax: { refuse: 'the rule ax needs formula 0 without other formulas in the goal' } },
    's3 3 0': { ax: 's4' }, 's3 3 1': { ax: 's4' }, 's4 4 0': { ax: 's5' }, 's4 4 1': { ax: 's5' },
  },
  close: {}, closeAll: {},
  exports: { svg: 'ill/derivation.svg', tex: 'ill/derivation.tex', typ: 'ill/derivation.typ', json: 'ill/interact/proof.json', v: 'ill/interact/proof.v', txt: 'ill/prove.txt' },
};

P.balance = {
  title: 'A ⊗ B ⊢ A', input: 'A * B |- A', uni: 'A ⊗ B ⊢ A', mode: 'classical', frag: 'MLL', engine: 'focus', dir: 'unprovable-balance',
  net: null, netWhy: 'The net appears when a proof is finished; this sequent has none.',
  states: { s0: { svg: 'unprovable-balance/sequent.svg', goals: ['goal 0: ⊢ 0: ~A ⅋ ~B, 1: A'] } },
  moves: { 's0 0 1': { ax: { refuse: 'the axiom needs formula 1 together with its dual literal and nothing else' } }, 's0 0 0': { '⅋': { unscripted: true } } },
  close: { 's0 0': { verdict: 'unbalanced', text: '~B occurs 1 more time than B in the one-sided sequent, so they cannot all meet in axioms', atom: 'B', extra: '~B', seq: '⊢ ~A ⅋ ~B, A' } },
  closeAll: {},
  exports: { cert: 'unprovable-balance/certificate.v', json: 'unprovable-balance/prove.json', txt: 'unprovable-balance/prove.txt', tex: 'unprovable-balance/derivation.tex', typ: 'unprovable-balance/derivation.typ' },
};
P.classical = {
  title: 'A ⊕ B ⊢ A & B', input: 'A + B |- A & B', uni: 'A ⊕ B ⊢ A & B', mode: 'classical', frag: 'ALL', engine: 'additive', dir: 'unprovable-classical',
  net: null, netWhy: 'No net view: proof nets are drawn for MLL, and this sequent has additives.',
  states: { s0: { svg: 'unprovable-classical/sequent.svg', goals: ['goal 0: ⊢ 0: ~A & ~B, 1: A & B'] } },
  moves: {},
  close: { 's0 0': { verdict: 'classical', text: 'the classical reading is false where B is true and every other atom false (! and ? erased, ⊗ and & as and, ⅋ and ⊕ as or, 1 and ⊤ true, ⊥ and 0 false), and every provable sequent reads as a tautology', atoms: ['A', 'B'], assignment: [false, true] } },
  closeAll: {},
  exports: { cert: 'unprovable-classical/certificate.v', json: 'unprovable-classical/prove.json', txt: 'unprovable-classical/prove.txt' },
};
P.exhaustive = {
  title: 'A & B ⊢ A ⊗ B', input: 'A & B |- A * B', uni: 'A & B ⊢ A ⊗ B', mode: 'classical', frag: 'MALL', engine: 'focus', dir: 'unprovable-exhaustive',
  net: null, netWhy: 'No net view: proof nets are drawn for MLL, and this sequent has additives.',
  states: { s0: { svg: 'unprovable-exhaustive/sequent.svg', goals: ['goal 0: ⊢ 0: ~A ⊕ ~B, 1: A ⊗ B'] } },
  moves: {},
  close: { 's0 0': { verdict: 'exhausted', text: 'the search was exhaustive' } },
  closeAll: {},
  exports: { nocert: 'unprovable-exhaustive/certificate.v.stderr', json: 'unprovable-exhaustive/prove.json', txt: 'unprovable-exhaustive/prove.txt' },
};
P.unknown = {
  title: '!(A ⊸ A ⊗ A), !(A ⊸ B), !(B ⊗ B ⊸ C), A ⊢ C ⊗ ?D', input: '!(A -o A * A), !(A -o B), !(B * B -o C), A |- C * ?D',
  uni: '!(A ⊸ A ⊗ A), !(A ⊸ B), !(B ⊗ B ⊸ C), A ⊢ C ⊗ ?D', mode: 'classical', frag: 'MELL', engine: 'focus', dir: 'unknown',
  net: null, netWhy: 'No net view for this sequent: it has exponentials.',
  states: { s0: { synth: '⊢ ?(A ⊗ (~A ⅋ ~A)), ?(A ⊗ ~B), ?((B ⊗ B) ⊗ ~C), ~A, C ⊗ ?D', goals: ['goal 0: ⊢ 0: ?(A ⊗ (~A ⅋ ~A)), 1: ?(A ⊗ ~B), 2: ?((B ⊗ B) ⊗ ~C), 3: ~A, 4: C ⊗ ?D'] } },
  moves: {},
  close: { 's0 0': { verdict: 'unknown', limit: 100000, after: '4.34 ms', copies: 11 } },
  closeAll: {},
  exports: { json: 'unknown/prove-work.json', txt: 'unknown/prove-work.txt' },
};
P.wide = {
  title: 'A ⊗ B ⊗ C ⊗ D ⊢ D ⊗ C ⊗ B ⊗ A', input: 'A * B * C * D |- D * C * B * A', uni: 'A ⊗ B ⊗ C ⊗ D ⊢ D ⊗ C ⊗ B ⊗ A', mode: 'classical', frag: 'MLL', engine: 'focus', dir: 'mll-wide',
  net: 'mll-wide/net.svg', netOk: true,
  states: { s0: { svg: 'mll-wide/derivation.svg', goals: [], done: true, search: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] } },
  moves: {}, close: {}, closeAll: {},
  exports: { svg: 'mll-wide/derivation.svg', txt: 'mll-wide/prove.txt', netsvg: 'mll-wide/net.svg' },
};
P.ordinary = {
  title: '((a → b) → a) → a', input: '((a -> b) -> a) -> a', uni: '((a → b) → a) → a', mode: 'classical logic', frag: 'MALL', engine: 'focus', dir: 'ordinary-classical', ordinary: true,
  net: null, netWhy: 'Ordinary logic has no net view.',
  states: { s0: { svg: 'ordinary-classical/derivation.svg', goals: [], done: true } },
  moves: {}, close: {}, closeAll: {},
  exports: { svg: 'ordinary-classical/derivation.svg', tex: 'ordinary-classical/derivation.tex', typ: 'ordinary-classical/derivation.typ', json: 'ordinary-classical/prove.json', v: 'ordinary-classical/certificate.v', txt: 'ordinary-classical/prove.txt' },
};


// Rules by connective and mode alone, as Interactive::rules lists them; two-sided
// names as core/src/proofs/rule.rs gives them.
function topConnective(f) {
  let depth = 0, best = null;
  const s = f.trim();
  for (let i = 0; i < s.length; i++) {
    const ch = s[i];
    if (ch === '(') depth++; else if (ch === ')') depth--;
    else if (depth === 0 && '⊗⅋&⊕⊸→'.includes(ch)) { if (!best || ch === '⊸' || ch === '→') best = ch; }
  }
  if (best) return best;
  if (s[0] === '(') return topConnective(s.slice(1, -1));
  if (s[0] === '!' || s[0] === '?') return s[0];
  if (s === '1' || s === '⊥' || s === '⊤' || s === '0') return s;
  if (s[0] === '~') return 'dual';
  return 'atom';
}
function rulesFor(f, side) {
  const c = topConnective(f);
  let r;
  if (app.mode === 'intuitionistic') {
    const L = side === 'L';
    r = ({ atom: [['ax']], '1': [[L ? '1L' : '1R']], '&': L ? [['&L₁'], ['&L₂']] : [['&R']], '⊕': L ? [['⊕L']] : [['⊕R₁'], ['⊕R₂']],
      '⊸': [[L ? '⊸L' : '⊸R', L ? 'split' : '']], '⊗': [[L ? '⊗L' : '⊗R', L ? '' : 'split']], '!': L ? [['!L'], ['!c'], ['!w']] : [['!R']], '⊤': [['⊤R']], '0': [['0L']] })[c] || [['ax']];
  } else {
    r = ({ atom: [['ax']], dual: [['ax']], '⊗': [['⊗', 'split']], '⅋': [['⅋']], '1': [['1']], '⊥': [['⊥']], '&': [['&']], '⊕': [['⊕₁'], ['⊕₂']], '⊤': [['⊤']], '0': [], '!': [['!']], '?': [['?d'], ['?c'], ['?w']] })[c] || [];
    if (app.affine && c !== '?') r = r.concat([['wk']]);
  }
  if (app.mix) r = r.concat([['Mix', 'split']]);
  return r;
}

// ---------- the reference (learning content, CC BY in the real site)
// one-sided rule: [premises, conclusion]; two-sided names with their schemas
const RULES = [
  { r: 'ax', one: ['', '⊢ ~A, A'], two: [['ax', '', 'A ⊢ A']] },
  { r: '⊗', one: ['⊢ Γ, A    ⊢ Δ, B', '⊢ Γ, Δ, A ⊗ B'], two: [['⊗R', 'Γ ⊢ A    Δ ⊢ B', 'Γ, Δ ⊢ A ⊗ B'], ['⊸L', 'Γ ⊢ A    Δ, B ⊢ C', 'Γ, Δ, A ⊸ B ⊢ C']] },
  { r: '⅋', one: ['⊢ Γ, A, B', '⊢ Γ, A ⅋ B'], two: [['⊸R', 'Γ, A ⊢ B', 'Γ ⊢ A ⊸ B'], ['⊗L', 'Γ, A, B ⊢ C', 'Γ, A ⊗ B ⊢ C']] },
  { r: '&', one: ['⊢ Γ, A    ⊢ Γ, B', '⊢ Γ, A & B'], two: [['&R', 'Γ ⊢ A    Γ ⊢ B', 'Γ ⊢ A & B'], ['⊕L', 'Γ, A ⊢ C    Γ, B ⊢ C', 'Γ, A ⊕ B ⊢ C']] },
  { r: '⊕₁', one: ['⊢ Γ, A', '⊢ Γ, A ⊕ B'], two: [['⊕R₁', 'Γ ⊢ A', 'Γ ⊢ A ⊕ B'], ['&L₁', 'Γ, A ⊢ C', 'Γ, A & B ⊢ C']] },
  { r: '⊕₂', one: ['⊢ Γ, B', '⊢ Γ, A ⊕ B'], two: [['⊕R₂', 'Γ ⊢ B', 'Γ ⊢ A ⊕ B'], ['&L₂', 'Γ, B ⊢ C', 'Γ, A & B ⊢ C']] },
  { r: '!', one: ['⊢ ?Γ, A', '⊢ ?Γ, !A'], two: [['!R', '!Γ ⊢ A', '!Γ ⊢ !A']] },
  { r: '?d', one: ['⊢ Γ, A', '⊢ Γ, ?A'], two: [['!L', 'Γ, A ⊢ C', 'Γ, !A ⊢ C']] },
  { r: '?c', one: ['⊢ Γ, ?A, ?A', '⊢ Γ, ?A'], two: [['!c', 'Γ, !A, !A ⊢ C', 'Γ, !A ⊢ C']] },
  { r: '?w', one: ['⊢ Γ', '⊢ Γ, ?A'], two: [['!w', 'Γ ⊢ C', 'Γ, !A ⊢ C']] },
  { r: '1', one: ['', '⊢ 1'], two: [['1R', '', '⊢ 1']] },
  { r: '⊥', one: ['⊢ Γ', '⊢ Γ, ⊥'], two: [['1L', 'Γ ⊢ C', 'Γ, 1 ⊢ C']] },
  { r: '⊤', one: ['', '⊢ Γ, ⊤'], two: [['⊤R', '', 'Γ ⊢ ⊤'], ['0L', '', 'Γ, 0 ⊢ C']] },
];
const RULE_BY_NAME = {};
RULES.forEach(R => { RULE_BY_NAME[R.r] = { name: R.r, prem: R.one[0], concl: R.one[1] }; R.two.forEach(([n, p, c]) => { if (!RULE_BY_NAME[n]) RULE_BY_NAME[n] = { name: n, prem: p, concl: c }; }); });
const CONNECTIVES = [
  ['⊗', 'times', 'both, side by side; each half uses its own resources', '⅋'],
  ['⅋', 'par', 'one or the other, the context decides; ~A ⅋ B is A ⊸ B', '⊗'],
  ['&', 'with', 'both offered, one taken, by the one who uses it', '⊕'],
  ['⊕', 'plus', 'one of two, chosen by the one who proves it', '&'],
  ['!', 'of course', 'as many copies as needed, or none', '?'],
  ['?', 'why not', 'the dual of !: may be copied, used once or dropped', '!'],
  ['⊸', 'lollipop', 'consumes A to produce B; A ⊸ B is ~A ⅋ B', ''],
  ['1', 'one', 'the unit of ⊗: nothing', '⊥'],
  ['⊥', 'bottom', 'the unit of ⅋', '1'],
  ['⊤', 'top', 'the unit of &: anything at all', '0'],
  ['0', 'zero', 'the unit of ⊕: no proof introduces it', '⊤'],
  ['A⊥', 'dual', 'negation; ~A in ASCII, (A⊥)⊥ is A', ''],
];
const READING = { '⊗': 0, '⅋': 1, '&': 2, '⊕': 3, '!': 4, '?': 5, '⊸': 6, '1': 7, '⊥': 8, '⊤': 9, '0': 10, dual: 11, atom: -1 };
function schemaHTML(prem, concl, name) {
  return '<span class="schema" role="img" aria-label="' + esc(name + ': from ' + (prem || 'nothing') + ' infer ' + concl) + '"><span>' + mathRaw(prem || ' ') + '</span><span class="bar"><span>' + esc(name) + '</span></span><span>' + mathRaw(concl) + '</span></span>';
}
function mathRaw(s) { return mathHTML(s).replace(/^<span class="m">|<\/span>$/g, ''); }

const EXP_REASONS = [
  { rule: '?c', opts: ['?d', '?c', '⊗'], why: '?~A is needed twice: once against A, once beside !A. Copy it first.' },
  { rule: '⊗', opts: ['⊗', '!', '?w'], why: 'One copy goes left with A, the other right with !A.' },
  { rule: '?d', opts: ['ax', '?d', '?c'], why: 'Drop the ? to use ~A once, against A.' },
  { rule: 'ax', opts: ['ax', '?w', '!'], why: '~A and A alone: an axiom.' },
  { rule: '!', opts: ['?d', '!', 'ax'], why: '! applies: the only other formula, ?~A, is a ? formula.' },
  { rule: '?d', opts: ['?w', '?d', '?c'], why: 'Again use ~A once.' },
  { rule: 'ax', opts: ['ax', '?d', '⊗'], why: '~A and A alone: an axiom.' },
];
const EXP_PATH = [['s0', 0, 0, '?c'], ['s1', 1, 2, '⊗', [0]], ['s2', 2, 0, '?d'], ['s3', 4, 0, 'ax'], ['s4', 3, 1, '!'], ['s5', 5, 0, '?d'], ['s6', 6, 0, 'ax']];

// ---------- application state
const app = {
  part: 'linear', mode: 'classical', mix: false, affine: false,
  proof: null, sk: null, active: null, hist: [], redo: [], view: 'tree',
  cam: { x: 0, y: 0, k: 1 }, sel: null, split: null, hint: 0, learn: false, rubric: 'tutorial', lesson: null, reason: null,
  verdict: null, prevDesc: null, worked: null, budget: 100000, retry: false, dockLine: null, menu: null,
  tabs: [], cur: -1, labels: true, searchLight: true,
};
const proof = () => P[app.proof];
const state = () => proof() && proof().states[app.sk];

// ---------- the stage
const stage = $('#stage'), world = $('#world'), drawing = $('#drawing'), hits = $('#hits');
let geo = null; // { w, h, goals: {n: {box, fs:[{box}]}}, conc: {id: box} }

function parseDesc(svg) {
  const d = svg.querySelector('desc');
  const out = {};
  if (!d) return out;
  d.textContent.split('\n').forEach(line => {
    const m = line.match(/^(\d+)\. (.*?), (open|by (.*?))\.$/);
    if (m) {
      const rule = m[4] ? m[4].replace(/ from .*$/, '') : null;
      out['i' + (m[1] - 1)] = { idx: +m[1], seq: m[2], open: m[3] === 'open', rule };
    }
  });
  return out;
}

function synthSvg(seq) {
  // Drawn by the mockup: the data has no SVG of this goal (see the README).
  let t = '';
  for (let i = 0; i < seq.length; i++) {
    const ch = seq[i];
    if (ch === '~' && /[A-Za-z]/.test(seq[i + 1])) { t += italic(seq[i + 1]) + '<tspan font-size="700" dy="-400">⊥</tspan><tspan dy="400">​</tspan>'; i++; }
    else t += /[A-Za-z]/.test(ch) ? italic(ch) : esc(ch);
  }
  return '<svg xmlns="http://www.w3.org/2000/svg" role="img" viewBox="0 0 30000 1700" width="480" height="27.2" font-family="\'Euler Math\', \'Neo Euler\', serif" font-size="1000" fill="black"><title>' + esc(seq) + '</title><desc>1. ' + esc(seq) + ', open.</desc><text id="i0" x="300" y="1190">' + t + '</text></svg>';
}

function charBoxes(el) {
  const texts = el.tagName === 'text' ? [el] : $$('text', el);
  const out = [];
  texts.forEach(t => {
    const s = t.textContent;
    let i = 0;
    while (i < s.length) {
      const cp = s.codePointAt(i), len = cp > 0xFFFF ? 2 : 1;
      let b = null;
      try { const r = t.getExtentOfChar(i); b = { x: r.x, y: r.y, w: r.width, h: r.height }; } catch (e) { b = null; }
      out.push({ ch: String.fromCodePoint(cp), b });
      i += len;
    }
  });
  return out;
}
function formulaBoxes(el) {
  const cs = charBoxes(el);
  const forms = [];
  let cur = [], depth = 0;
  const flush = () => { const c = cur.filter(x => x.b && x.ch.trim() && x.ch !== '​'); if (c.length) forms.push(c); cur = []; };
  cs.forEach(c => {
    if (c.ch === '(') depth++;
    if (c.ch === ')') depth--;
    if (depth === 0 && (c.ch === ',' || c.ch === '⊢')) flush(); else cur.push(c);
  });
  flush();
  return forms.map(c => {
    const x0 = Math.min(...c.map(q => q.b.x)), x1 = Math.max(...c.map(q => q.b.x + q.b.w));
    const y0 = Math.min(...c.map(q => q.b.y)), y1 = Math.max(...c.map(q => q.b.y + q.b.h));
    return { x: x0, y: y0, w: x1 - x0, h: y1 - y0 };
  });
}

function drawState(animate) {
  const st = state();
  const pf = proof();
  drawing.innerHTML = '';
  hits.innerHTML = '';
  let src;
  if (app.view === 'net') src = D[pf.net];
  else src = st.svg ? D[st.svg] : synthSvg(st.synth);
  drawing.innerHTML = src.replace(/<!--.*?-->/s, '');
  const svg = drawing.querySelector('svg');
  svg.removeAttribute('role');
  svg.setAttribute('aria-hidden', 'true');
  const vb = svg.viewBox.baseVal;
  if (st.synth && app.view === 'tree') {
    const t = svg.querySelector('text');
    const w = t.getBBox().width + 600;
    svg.setAttribute('viewBox', '0 0 ' + w + ' 1700');
  }
  const W = vb.width * SCALE, H = vb.height * SCALE;
  svg.setAttribute('width', W); svg.setAttribute('height', H);
  world.style.width = W + 'px'; world.style.height = H + 'px';
  geo = { w: W, h: H, goals: {}, conc: {}, labels: [] };
  if (app.view === 'net') {
    $$('g[stroke="#0072b2"]', svg).forEach(g => g.classList.add('netpar'));
    $('#drawing').setAttribute('aria-label', 'Proof net');
    return;
  }
  // conclusions, labels, dots
  if (!svg.querySelector('[id^="i"]')) { const g = svg.querySelector('g:not([stroke]), text'); if (g) g.id = 'i0'; }
  const desc = parseDesc(svg);
  const goals = st.goals.map(parseGoal);
  if (!Object.keys(desc).length && goals.length === 1) desc.i0 = { idx: 1, seq: goals[0].seq, open: true, rule: null };
  const used = new Set();
  let last = null;
  Array.from(svg.children).forEach(ch => {
    if (ch.id && /^i\d+$/.test(ch.id)) {
      last = ch.id;
      const info = desc[ch.id];
      const bb = ch.getBBox();
      geo.conc[ch.id] = { x: bb.x * SCALE, y: bb.y * SCALE, w: bb.width * SCALE, h: bb.height * SCALE, info };
      if (info && info.open) {
        const g = goals.find(q => q.seq === info.seq && !used.has(q.n));
        if (g) {
          used.add(g.n);
          ch.classList.add('open-goal'); ch.dataset.goal = g.n;
          geo.goals[g.n] = { el: ch, box: geo.conc[ch.id], fs: formulaBoxes(ch).map(b => ({ x: b.x * SCALE, y: b.y * SCALE, w: b.w * SCALE, h: b.h * SCALE })), g };
        }
      }
    } else if (ch.tagName === 'text' && ch.textContent === '⋮') {
      ch.classList.add('dots'); ch.dataset.of = last;
      const G = Object.values(geo.goals).find(q => q.el.id === last);
      if (G) G.dotsY = ch.getBBox().y * SCALE;
    } else if ((ch.tagName === 'text' || ch.tagName === 'g') && ch.getAttribute('font-size') === '800') {
      ch.classList.add('lbl'); ch.dataset.of = last;
      geo.labels.push(ch);
    }
  });
  // the search's inferences (D10)
  if (st.search) {
    const bars = $$('g[stroke] path', svg).map(p => { const m = p.getAttribute('d').match(/M(\d+) (\d+)H(\d+)/); return m ? { p, x0: +m[1], y: +m[2], x1: +m[3] } : null; }).filter(Boolean);
    geo.labels.forEach(l => {
      const info = desc[l.dataset.of];
      if (info && st.search.includes(info.idx)) {
        l.classList.add('search');
        const t = l.tagName === 'text' ? l : l.querySelector('text');
        const lx = +t.getAttribute('x'), ly = +t.getAttribute('y');
        const bar = bars.find(b => Math.abs(b.y - (ly - 200)) < 60 && lx >= b.x1 - 10 && lx <= b.x1 + 400);
        if (bar) bar.p.classList.add('bar-search');
      }
    });
  }
  // motion: what is new since the previous state
  if (animate && app.prevDesc && !reducedMQ.matches) {
    const before = {};
    Object.values(app.prevDesc).forEach(v => { const k = v.seq + '|' + v.rule; before[k] = (before[k] || 0) + 1; });
    Array.from(svg.children).forEach(ch => {
      if (ch.classList.contains('lbl')) {
        const info = desc[ch.dataset.of];
        const k = info && info.seq + '|' + info.rule;
        if (k && before[k]) before[k]--; else ch.classList.add('settle');
      } else if (ch.classList.contains('open-goal') || ch.classList.contains('dots')) {
        const info = desc[ch.id || ch.dataset.of];
        const k = info && info.seq + '|null';
        if (ch.id) { if (k && before[k]) before[k]--; else ch.classList.add('arrive'); }
      }
    });
    $$('.dots', svg).forEach(d => { const c = svg.querySelector('#' + d.dataset.of); if (c && c.classList.contains('arrive')) d.classList.add('arrive'); });
  }
  app.prevDesc = desc;
  buildHits();
  markActive();
}

function buildHits() {
  hits.innerHTML = '';
  Object.values(geo.goals).forEach(G => {
    G.fs.forEach((b, p) => {
      const f = G.g.fs[p];
      if (f === undefined) return;
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = 'fb';
      btn.dataset.goal = G.g.n; btn.dataset.pos = p;
      const w = Math.max(24, b.w + 8), h = Math.max(24, b.h + 4);
      btn.style.left = (b.x + b.w / 2 - w / 2) + 'px';
      btn.style.top = (b.y + b.h / 2 - h / 2) + 'px';
      btn.style.width = w + 'px'; btn.style.height = h + 'px';
      btn.setAttribute('aria-label', f + ', formula ' + p + ' of goal ' + G.g.n);
      btn.setAttribute('aria-haspopup', 'dialog');
      const ri = READING[topConnective(f)];
      if (ri !== undefined && ri >= 0) btn.dataset.ltip = mathHTML(CONNECTIVES[ri][0]) + ' ' + esc(CONNECTIVES[ri][1]) + ': ' + fmtText(CONNECTIVES[ri][2]);
      else btn.dataset.ltip = 'an atom: closed by ax against its dual';
      btn.addEventListener('click', ev => { ev.stopPropagation(); onFormula(G.g.n, p, btn); });
      hits.appendChild(btn);
    });
  });
}

function markActive() {
  const svg = drawing.querySelector('svg');
  if (!svg) return;
  const many = Object.keys(geo.goals).length > 1;
  Object.values(geo.goals).forEach(G => {
    const on = G.g.n === app.active;
    G.el.classList.toggle('active', on);
    G.el.classList.toggle('dim', many && !on);
    const dots = svg.querySelector('.dots[data-of="' + G.el.id + '"]');
    if (dots) { dots.classList.toggle('active', on); dots.classList.toggle('dim', many && !on); }
  });
  $$('.fb', hits).forEach(b => b.classList.toggle('dim', many && +b.dataset.goal !== app.active && !app.split));
  renderDock();
}

// ---------- camera
function pad() {
  // room above the active goal for its dock, and below for the stage's own controls
  if (phoneMQ.matches) return { t: 70, b: 52, l: 12, r: 12 };
  return { t: 80, b: 70, l: 40, r: 40 };
}
function applyCam(glide) {
  if (DEBUG) document.body.dataset.cam = JSON.stringify({ cam: app.cam, st: stage.getBoundingClientRect(), geo: geo && [geo.w, geo.h], vw: innerWidth });
  world.classList.toggle('glide', !!glide && !reducedMQ.matches);
  world.style.transform = 'translate(' + app.cam.x + 'px,' + app.cam.y + 'px) scale(' + app.cam.k + ')';
  $('#zoom-v').textContent = Math.round(app.cam.k * 100) + '%';
  stage.style.backgroundSize = (24 * app.cam.k) + 'px ' + (24 * app.cam.k) + 'px';
  stage.style.backgroundPosition = app.cam.x + 'px ' + app.cam.y + 'px';
  layoutDock();
  updateMinimap();
}
function fit(glide) {
  if (!geo) return;
  const r = stage.getBoundingClientRect(), p = pad();
  const aw = Math.max(80, r.width - p.l - p.r), ah = Math.max(80, r.height - p.t - p.b);
  const k = Math.max(0.3, Math.min(1.4, aw / geo.w, ah / geo.h));
  app.cam = { k, x: p.l + (aw - geo.w * k) / 2, y: p.t + (ah - geo.h * k) / 2 };
  applyCam(glide);
}
function focusGoal(glide) {
  const G = geo && geo.goals[app.active];
  if (!G) return fit(glide);
  const r = stage.getBoundingClientRect(), p = pad();
  let k = app.cam.k;
  if (k < 0.9) k = Math.min(1.2, Math.max(0.9, k));
  const aw = r.width - p.l - p.r, ah = r.height - p.t - p.b;
  // keep the whole tree in view when it fits at this zoom
  if (geo.w * k <= aw && geo.h * k <= ah) { app.cam = { k, x: p.l + (aw - geo.w * k) / 2, y: p.t + (ah - geo.h * k) / 2 }; }
  else {
    const cx = G.box.x + G.box.w / 2, cy = G.box.y + G.box.h / 2;
    app.cam = { k, x: p.l + aw / 2 - cx * k, y: p.t + ah * 0.45 - cy * k };
  }
  applyCam(glide);
}
function zoomAt(f, sx, sy) {
  const r = stage.getBoundingClientRect();
  if (sx === undefined) { sx = r.width / 2; sy = r.height / 2; }
  const k = Math.max(0.2, Math.min(4, app.cam.k * f));
  app.cam.x = sx - (sx - app.cam.x) * (k / app.cam.k);
  app.cam.y = sy - (sy - app.cam.y) * (k / app.cam.k);
  app.cam.k = k;
  applyCam(true);
}
function updateMinimap() {
  const mm = $('#minimap'), inner = $('#mm-in'), view = $('#mm-view');
  if (!geo || !drawing.querySelector('svg')) { mm.hidden = true; return; }
  mm.hidden = false;
  if (inner.dataset.src !== app.proof + app.sk + app.view) {
    inner.innerHTML = drawing.innerHTML;
    const s = inner.querySelector('svg');
    if (s) { s.removeAttribute('width'); s.removeAttribute('height'); s.style.width = '100%'; s.style.height = '100%'; }
    inner.dataset.src = app.proof + app.sk + app.view;
  }
  const mw = 120 - 6, mh = 40 - 6;
  const s = Math.min(mw / geo.w, mh / geo.h);
  const ox = 3 + (mw - geo.w * s) / 2, oy = 3 + (mh - geo.h * s) / 2;
  const r = stage.getBoundingClientRect();
  const vx = -app.cam.x / app.cam.k, vy = -app.cam.y / app.cam.k, vw = r.width / app.cam.k, vh = r.height / app.cam.k;
  const L = Math.max(0, ox + vx * s), T = Math.max(0, oy + vy * s);
  const R = Math.min(120, ox + (vx + vw) * s), B = Math.min(40, oy + (vy + vh) * s);
  Object.assign(view.style, { left: L + 'px', top: T + 'px', width: Math.max(4, R - L) + 'px', height: Math.max(4, B - T) + 'px' });
}

// pan by pointer (optional: the buttons and keys do the same), zoom by wheel
(function () {
  let drag = null;
  stage.addEventListener('pointerdown', e => {
    if (e.target.closest('.fb, .cluster, button, a, input, textarea, summary')) return;
    drag = { x: e.clientX, y: e.clientY, cx: app.cam.x, cy: app.cam.y, moved: false, id: e.pointerId };
  });
  stage.addEventListener('pointermove', e => {
    if (!drag || e.pointerId !== drag.id) return;
    const dx = e.clientX - drag.x, dy = e.clientY - drag.y;
    if (!drag.moved && Math.hypot(dx, dy) > 4) { drag.moved = true; stage.classList.add('panning'); stage.setPointerCapture(e.pointerId); }
    if (drag.moved) { app.cam.x = drag.cx + dx; app.cam.y = drag.cy + dy; applyCam(false); }
  });
  const end = e => { if (drag && !drag.moved && !e.target.closest('.pop')) { closePop(); if (app.dockLine && app.dockLine.kind === 'refusal') setDockLine(null); } drag = null; stage.classList.remove('panning'); };
  stage.addEventListener('pointerup', end);
  stage.addEventListener('pointercancel', () => { drag = null; stage.classList.remove('panning'); });
  stage.addEventListener('wheel', e => {
    e.preventDefault();
    const r = stage.getBoundingClientRect();
    if (e.ctrlKey || Math.abs(e.deltaY) > Math.abs(e.deltaX)) zoomAt(e.deltaY < 0 ? 1.1 : 1 / 1.1, e.clientX - r.left, e.clientY - r.top);
    else { app.cam.x -= e.deltaX; applyCam(false); }
  }, { passive: false });
  // two-finger pinch
  const pts = new Map(); let pinch = null;
  stage.addEventListener('touchstart', e => { if (e.touches.length === 2) { const [a, b] = e.touches; pinch = { d: Math.hypot(a.clientX - b.clientX, a.clientY - b.clientY), k: app.cam.k }; } }, { passive: true });
  stage.addEventListener('touchmove', e => {
    if (pinch && e.touches.length === 2) {
      const [a, b] = e.touches, d = Math.hypot(a.clientX - b.clientX, a.clientY - b.clientY);
      const r = stage.getBoundingClientRect();
      zoomAt((pinch.k * d / pinch.d) / app.cam.k, (a.clientX + b.clientX) / 2 - r.left, (a.clientY + b.clientY) / 2 - r.top);
    }
  }, { passive: true });
  stage.addEventListener('touchend', () => { pinch = null; pts.clear(); });
})();

// ---------- split selection: the goal's formulas become left/right toggles
function startSplit(s, r, spec) {
  const G = geo.goals[s.goal].g;
  app.split = { goal: s.goal, pos: s.pos, rule: r, left: new Set(), spec: spec || null, G };
  $$('.fb').forEach(b => {
    const g = +b.dataset.goal, p = +b.dataset.pos;
    if (g !== s.goal) { b.disabled = true; b.classList.add('dim'); }
    else if (p === s.pos) { b.classList.add('principal'); b.classList.remove('sel'); b.setAttribute('aria-pressed', 'mixed'); }
    else { b.setAttribute('aria-pressed', 'false'); b.setAttribute('aria-label', G.fs[p] + ': send to the left premise'); }
  });
  renderSplitPop();
}
function exitSplit() {
  app.split = null;
  $$('.fb').forEach(b => { b.disabled = false; b.classList.remove('left', 'principal', 'dim'); b.removeAttribute('aria-pressed'); });
  markActive();
}
function toggleSplit(goal, pos, btn) {
  const sp = app.split;
  if (goal !== sp.goal || pos === sp.pos) return;
  if (sp.left.has(pos)) sp.left.delete(pos); else sp.left.add(pos);
  btn.classList.toggle('left', sp.left.has(pos));
  btn.setAttribute('aria-pressed', String(sp.left.has(pos)));
  renderSplitPop();
}
function splitPremises(sp) {
  // positional, as the library opens them: the principal's halves take its place
  const G = sp.G, f = G.fs[sp.pos];
  const m = f.match(/^(.*) (⊗|⊸) (.*)$/);
  const a = m ? m[1] : f, b = m ? m[3] : '';
  if (G.two) {
    const lh = [], ls = [], rh = [], rs = [];
    G.fs.forEach((x, i) => {
      const hyp = i < G.nLeft;
      if (i === sp.pos) { rh.push(b); return; }
      if (sp.left.has(i)) (hyp ? lh : ls).push(x); else (hyp ? rh : rs).push(x);
    });
    return [lh.join(', ') + ' ⊢ ' + ls.concat([a]).join(', '), rh.join(', ') + ' ⊢ ' + rs.join(', ')];
  }
  const L = [], R = [];
  G.fs.forEach((x, i) => {
    if (i === sp.pos) { L.push(a); R.push(b); return; }
    (sp.left.has(i) ? L : R).push(x);
  });
  return ['⊢ ' + L.join(', '), '⊢ ' + R.join(', ')];
}
function splitResult(sp) {
  if (!sp.spec) return null;
  const key = Array.from(sp.left).sort().join(',');
  return sp.spec.res[key] || null;
}
function applySplit() {
  const sp = app.split; if (!sp) return;
  const res = splitResult(sp);
  if (!res) return;
  if (res.refuse) { refuse(sp.goal, sp.pos, res.refuse, sp.rule); closePop(); return; }
  if (!res.to) { notScripted(); return; }
  const left = Array.from(sp.left).sort();
  closePop();
  step({ to: res.to, goal: sp.goal, pos: sp.pos, rule: sp.rule, left });
}


// ---------- one hover rule: [data-tip] (and, inside the Learn frame, [data-ltip]); the key rides in the tip (D55)
const tipEl = $('#tip');
let tipFor = null;
const TIPSEL = '[data-tip],[data-tip-html],[data-ltip],[data-key]';
function tipContent(el) {
  const key = el.dataset.key ? ' <kbd>' + esc(el.dataset.key) + '</kbd>' : '';
  if (el.dataset.tipHtml) return el.dataset.tipHtml + key;
  if (el.dataset.tip) return esc(el.dataset.tip) + key;
  if (app.learn && el.dataset.ltip) return el.dataset.ltip;
  if (key) return esc(el.getAttribute('aria-label') || el.textContent.trim()) + key;
  return null;
}
function showTip(el) {
  const html = tipContent(el); if (!html) return;
  tipFor = el;
  tipEl.innerHTML = html; tipEl.hidden = false;
  const r = el.getBoundingClientRect(), w = tipEl.offsetWidth, h = tipEl.offsetHeight;
  let x = r.left + r.width / 2 - w / 2, y = r.top - h - 8;
  if (y < 6 || r.top < 90) y = r.bottom + 8;
  tipEl.style.left = Math.max(6, Math.min(x, innerWidth - w - 6)) + 'px';
  tipEl.style.top = y + 'px';
}
function hideTip() { tipEl.hidden = true; tipFor = null; }
document.addEventListener('mouseover', e => { const el = e.target.closest(TIPSEL); if (el && el !== tipFor) showTip(el); else if (!el && tipFor) hideTip(); });
document.addEventListener('focusin', e => { const el = e.target.closest(TIPSEL); if (el && el.id !== 'field') showTip(el); else hideTip(); });
document.addEventListener('focusout', () => hideTip());

// ---------- the dock: everything that acts on the active goal, at the goal (D48)
const dockEl = $('#dock');
function setDockLine(line) { app.dockLine = line; renderDock(); }
function renderDock() {
  const st = state();
  if (!st || st.done || app.view !== 'tree' || !geo || !geo.goals[app.active]) { dockEl.hidden = true; return; }
  const goals = st.goals.map(g => parseGoal(g).n);
  const i = goals.indexOf(app.active);
  let h = '';
  const L = app.dockLine;
  if (L) h += '<div class="cluster dline">' + L.html + '</div>';
  h += '<div class="cluster gbar">' +
    (goals.length > 1 ? '<button type="button" class="btn btn-s" data-d="prev" aria-label="Previous open goal" data-tip="Previous goal" data-key="[">‹</button><span class="gn">Goal ' + app.active + ' (' + (i + 1) + ' of ' + goals.length + ' open)</span><button type="button" class="btn btn-s" data-d="next" aria-label="Next open goal" data-tip="Next goal" data-key="]">›</button><span class="sep"></span>' : '<span class="gn">Goal ' + app.active + '</span>') +
    '<button type="button" class="btn btn-s" data-d="close" data-tip="The search closes this goal" data-key="C"><svg class="ic" aria-hidden="true"><use href="#i-closeall"/></svg>Close goal<kbd class="k">C</kbd></button></div>';
  dockEl.innerHTML = h;
  dockEl.hidden = false;
  $$('[data-d]', dockEl).forEach(b => b.addEventListener('click', () => { const a = b.dataset.d; if (a === 'prev') cycleGoal(-1); if (a === 'next') cycleGoal(1); if (a === 'close') closeGoal(false); }));
  if (L && L.wire) L.wire(dockEl);
  layoutDock();
}
function layoutDock() {
  if (dockEl.hidden || !geo) { layoutReasons(); return; }
  const G = geo.goals[app.active]; if (!G) return;
  const k = app.cam.k, r = stage.getBoundingClientRect();
  const top = G.dotsY !== undefined ? Math.min(G.dotsY, G.box.y) : G.box.y;
  const h = dockEl.offsetHeight, w = dockEl.offsetWidth;
  let x = app.cam.x + G.box.x * k, y = app.cam.y + top * k - h - 6;
  if (y < 6) y = 6;
  dockEl.style.left = Math.max(8, Math.min(x, r.width - w - 8)) + 'px';
  dockEl.style.top = y + 'px';
  layoutReasons();
}
// reasons at an inference: learning content, only inside the Learn frame
function layoutReasons() {
  const box = $('#reasons');
  box.innerHTML = '';
  if (!app.learn || !app.reason || !geo) return;
  const c = geo.conc[app.reason.conc]; if (!c) return;
  const d = document.createElement('div');
  d.className = 'cluster reason';
  d.innerHTML = '<span class="kw-label" style="color:var(--learn)">Why ' + esc(app.reason.rule) + '</span><div>' + fmtText(app.reason.why) + '</div>';
  box.appendChild(d);
  const r = stage.getBoundingClientRect();
  let x = app.cam.x + geo.w * app.cam.k + 14;
  if (x + 260 > r.width) x = Math.max(8, r.width - 270);
  d.style.left = x + 'px'; d.style.top = (app.cam.y + c.y * app.cam.k) + 'px';
}

// ---------- goals, and the verdict: the active tab's dot, details on hover or click (D44, D53)
function setActive(n, glide) {
  app.active = n; app.hint = 0; updateHintRung(); app.dockLine = null;
  markActive(); focusGoal(glide);
  const g = state().goals.map(parseGoal).find(q => q.n === n);
  if (g) announce('Goal ' + n + ': ' + speak(g.seq));
}
function cycleGoal(d) {
  const st = state(); if (!st) return;
  const ns = st.goals.map(g => parseGoal(g).n); if (ns.length < 2) return;
  const i = ns.indexOf(app.active);
  setActive(ns[(i + d + ns.length) % ns.length], true);
}
function baseRows(pf, extra) {
  return [['Fragment', pf.frag], ['Calculus', calcName(pf.ordinary ? null : { mode: app.mode, mix: app.mix, affine: app.affine }, pf)], ['Engine', pf.engine]].concat(extra || []);
}
function verdictNow() {
  const pf = proof(), st = state();
  if (!pf) return { word: 'No proof', lamp: '' };
  if (app.verdict) return app.verdict;
  if (st.done) {
    const bySearch = st.search && st.search.length;
    return { lamp: 'ok', word: pf.ordinary ? 'Valid' : 'Proved', tone: 'ok', checked: true, rows: baseRows(pf, bySearch ? [['Search', 'its steps have lighter labels']] : []),
      reason: pf.ordinary ? 'Valid in classical logic: the translation into affine MALL was proved and checked.' : 'No goal is open; the library\'s checker accepted the proof.' };
  }
  const n = st.goals.length;
  return { lamp: 'open', word: n + ' open', tone: 'open', rows: baseRows(pf), reason: n + ' goal' + (n > 1 ? 's' : '') + ' still open.' };
}
function refreshStatus() { if (!$('#status-pop').hidden) renderStatusPop(); renderProofRow(); }
let statusPinned = false;
function activeDot() { return $('#proofs .ptab[aria-current="true"] .pt-dot'); }
function renderStatusPop() {
  const v = verdictNow(), sp = $('#status-pop');
  sp.innerHTML = '<p class="dl-h"><span class="lamp ' + (v.lamp ? 'on-' + v.lamp : '') + '"></span><span class="kw-label ' + (v.tone || '') + '" style="color:var(--' + ({ ok: 'ok', bad: 'bad', warn: 'warn', open: 'accent' }[v.tone] || 'ink') + ')">' + esc(v.word) + '</span>' + (v.checked ? '<span class="check">✓ Checked</span>' : (v.unchecked ? '<span class="kw plain">Not checked</span>' : '')) + '</p>' +
    (v.rows ? '<dl>' + v.rows.map(([k, x]) => '<dt>' + esc(k) + '</dt><dd>' + esc(x) + '</dd>').join('') + '</dl>' : '') +
    (v.reason ? '<p>' + v.reason + '</p>' : '') + (v.extra || '') +
    (v.actions ? '<div class="row" style="margin:6px 4px 2px">' + v.actions + '</div>' : '');
  wireActs(sp);
  const d = activeDot(); if (!d) { sp.hidden = true; return; }
  const r = d.getBoundingClientRect();
  sp.hidden = false;
  sp.style.top = (r.bottom + 8) + 'px';
  sp.style.left = Math.max(8, Math.min(r.left - 8, innerWidth - sp.offsetWidth - 8)) + 'px';
}
function openStatus(pin) { if (!proof()) return; statusPinned = !!pin; renderStatusPop(); const d = activeDot(); if (d) d.setAttribute('aria-expanded', 'true'); }
function closeStatus() { $('#status-pop').hidden = true; statusPinned = false; const d = activeDot(); if (d) d.setAttribute('aria-expanded', 'false'); }

// ---------- the rule popover, beside the goal so the goal stays visible (D9)
const pop = $('#pop');
function closePop() {
  if (pop.hidden) return;
  pop.hidden = true;
  $$('.fb.sel').forEach(b => b.classList.remove('sel'));
  if (app.split) exitSplit();
  app.sel = null;
}
function placePop(goal) {
  // to the right of the whole goal, else to its left, else above it
  pop.hidden = false;
  const G = geo.goals[goal]; const sr = stage.getBoundingClientRect(), k = app.cam.k;
  const gx0 = sr.left + app.cam.x + G.box.x * k, gx1 = sr.left + app.cam.x + (G.box.x + G.box.w) * k;
  const gy0 = sr.top + app.cam.y + G.box.y * k, gy1 = sr.top + app.cam.y + (G.box.y + G.box.h) * k;
  const pw = pop.offsetWidth, ph = pop.offsetHeight;
  let x, y = Math.max(sr.top + 6, Math.min((gy0 + gy1) / 2 - ph / 2, sr.bottom - ph - 6));
  if (gx1 + 14 + pw < sr.right - 6) x = gx1 + 14;
  else if (gx0 - 14 - pw > sr.left + 6) x = gx0 - 14 - pw;
  else { x = Math.max(sr.left + 6, Math.min(gx0, sr.right - pw - 6)); y = Math.max(sr.top + 6, gy0 - ph - 48); }
  pop.style.left = x + 'px'; pop.style.top = y + 'px';
}
function moveFor(goal, pos) { const pf = proof(); return (pf.moves || {})[app.sk + ' ' + goal + ' ' + pos] || {}; }
function onFormula(goal, pos, btn) {
  if (app.split) { toggleSplit(goal, pos, btn); return; }
  if (goal !== app.active) { app.active = goal; markActive(); }
  if (app.dockLine && app.dockLine.kind === 'refusal') setDockLine(null);
  $$('.fb.sel').forEach(b => b.classList.remove('sel'));
  btn.classList.add('sel');
  app.sel = { goal, pos, btn };
  const G = geo.goals[goal].g, f = G.fs[pos];
  const side = G.two && pos < G.nLeft ? 'L' : 'R';
  const rules = rulesFor(f, side);
  pop.setAttribute('aria-label', 'Rules for ' + f);
  pop.innerHTML = '<p class="pop-h"><span class="kw-label">Rules</span>' + mathHTML(f) + '</p>' +
    (rules.length ? '<ol class="rules">' + rules.map(([r, note], i) => {
      const S = RULE_BY_NAME[r];
      const lt = S ? ' data-ltip="' + esc(schemaHTML(S.prem, S.concl, r)) + '"' : '';
      return '<li><button type="button" data-r="' + esc(r) + '"' + lt + ' data-key="' + (i + 1) + '"><kbd class="k">' + (i + 1) + '</kbd><span class="rn">' + esc(r) + '</span><span class="rd">' + esc(note || '') + '</span></button></li>';
    }).join('') + '</ol>' : '<p class="pop-foot">No rule acts on 0.</p>');
  $$('.rules button', pop).forEach(b => b.addEventListener('click', () => pickRule(b.dataset.r)));
  placePop(goal);
  const first = $('.rules button', pop); if (first) first.focus();
}
function pickRule(r) {
  const s = app.sel; if (!s) return;
  const mv = moveFor(s.goal, s.pos);
  if (/^(⊗|⊸L|⊗R|Mix)$/.test(r)) { startSplit(s, r, mv.split); return; }
  const out = mv[r];
  if (!out || out.unscripted || out.refuseNote) { notScripted(); return; }
  if (out.refuse) { refuse(s.goal, s.pos, out.refuse, r); closePop(); return; }
  closePop();
  step({ to: out, goal: s.goal, pos: s.pos, rule: r });
}
function notScripted() {
  pop.querySelector('.mock') || pop.insertAdjacentHTML('beforeend', '<span class="mock">Legal, but not scripted in this mockup</span>');
}
function renderSplitPop() {
  const sp = app.split;
  const [l, r] = splitPremises(sp);
  const res = splitResult(sp);
  let v;
  if (!res) v = '<span class="kw plain" tabindex="0" data-tip="The mockup has no state for this split.">Not scripted</span>';
  else if (res.refuse) v = '<span class="kw bad" tabindex="0" data-tip="' + esc(sentence(res.refuse)) + '">Not allowed</span>';
  else if (res.pass) v = '<span class="kw acc" tabindex="0" data-tip="Passes the count test of the search; it may still fail later.">Passes</span>';
  else v = '<span class="kw plain" tabindex="0" data-tip="An atom\'s count does not balance in a premise, so this split cannot close. It is still legal.">Cannot close</span>';
  const fail = !res || !res.pass;
  pop.innerHTML = '<p class="pop-h"><span class="kw-label">Split for ' + ruleHTML(sp.rule) + '</span>' + mathHTML(sp.G.fs[sp.pos]) + '</p>' +
    '<p class="pop-foot" style="margin:0 4px 6px">Click the formulas that go left.</p>' +
    '<div class="split-pre' + (fail ? ' fail' : '') + '"><div class="prem"><span class="kw-label">Left</span>' + mathHTML(l) + '</div><div class="prem"><span class="kw-label">Right</span>' + mathHTML(r) + '</div></div>' +
    '<div class="row">' + v + '<span class="grow"></span><button type="button" class="btn btn-primary" id="split-go"' + (!res || res.refuse ? ' aria-disabled="true"' : '') + ' data-key="Enter">Apply ' + ruleHTML(sp.rule) + '</button><button type="button" class="btn btn-s" id="split-x" data-key="Esc">Cancel</button></div>';
  $('#split-go').addEventListener('click', applySplit);
  $('#split-x').addEventListener('click', closePop);
  placePop(sp.goal);
}

// a refusal: a keyword line at the goal, the full reason on hover or focus (D11, D44)
function sentence(msg) { return msg.charAt(0).toUpperCase() + msg.slice(1) + '.'; }
function refusalWords(goal, pos, msg) {
  const G = geo.goals[goal] && geo.goals[goal].g;
  const f = G ? G.fs[pos] : '';
  let m = msg.match(/^the axiom needs formula (\d+) together with its dual literal and nothing else$/);
  if (m && G) { const others = G.fs.filter((_, i) => i !== pos); return ['no dual partner', 'ax needs ' + tx(f) + ' together with its dual and nothing else; here it sits with ' + others.map(tx).join(', ') + '.']; }
  m = msg.match(/^the rule ax needs formula (\d+) without other formulas in the goal$/);
  if (m && G) return ['not alone', 'ax needs ' + tx(f) + ' alone in the goal; this goal has other formulas.'];
  if (/2 formulas on the right/.test(msg)) return ['two on the right', 'A premise would have 2 formulas right of ⊢; an intuitionistic sequent has one.'];
  return ['refused', sentence(msg)];
}
const tx = f => f.replace(/~([A-Za-z])/g, '$1⊥');
function refuse(goal, pos, msg, rule) {
  const btn = hits.querySelector('.fb[data-goal="' + goal + '"][data-pos="' + pos + '"]');
  if (btn && !reducedMQ.matches) { btn.classList.remove('shake'); void btn.offsetWidth; btn.classList.add('shake'); }
  const [kw, full] = refusalWords(goal, pos, msg);
  app.active = goal; markActive();
  setDockLine({ kind: 'refusal', html: '<button type="button" class="kw bad" data-tip="' + esc(full) + '" aria-label="' + esc(rule + ' refused: ' + full) + '">' + ruleHTML(rule) + ' refused: ' + esc(kw) + '</button>' });
  announce(rule + ' refused: ' + full);
}
const tmpText = h => { const d = document.createElement('div'); d.innerHTML = h; return d.textContent; };

// ---------- steps, undo per branch, closing
function step(o, quiet) {
  app.hist.push({ from: app.sk, to: o.to, goal: o.goal, pos: o.pos, rule: o.rule, left: o.left, by: o.by || 'you' });
  app.redo = [];
  go(o.to, true);
  if (!quiet) announce(state().done ? 'Applied ' + o.rule + '. Proof checked.' : 'Applied ' + o.rule + ' on goal ' + o.goal + '.');
  lessonWatch();
}
function go(sk, animate) {
  app.sk = sk;
  app.verdict = null; app.retry = false; app.dockLine = null;
  const st = state();
  const ns = st.goals.map(g => parseGoal(g).n);
  if (!ns.includes(app.active)) app.active = ns.length ? (ns.includes(app.active + 1) ? app.active + 1 : ns[0]) : null;
  if (app.view === 'net' && !st.done) app.view = 'tree';
  drawState(animate);
  refreshStatus();
  renderViewTabs();
  if (animate) focusGoal(true); else fit(false);
  updateUndo();
  if (!$('#menu').hidden && app.menu === 'steps') renderMenu('steps');
}
function undo() {
  const h = app.hist.pop(); if (!h) return;
  app.redo.push(h);
  app.active = h.goal;
  const goneFrom = proof().states[h.to].goals.map(g => parseGoal(g).n);
  go(h.from, true);
  const now = state().goals.map(g => parseGoal(g).n);
  const gone = goneFrom.filter(n => !now.includes(n));
  toast('Undone: ' + (h.rule === 'close' ? 'the search' : h.rule) + ' on goal ' + h.goal + (gone.length > 1 ? '; goals ' + gone.join(' and ') + ' removed' : ''));
  announce('Undid ' + h.rule + ': goal ' + h.goal + ' is open again.');
}
function redo() { const h = app.redo.pop(); if (!h) return; app.hist.push(h); go(h.to, true); toast('Redone: ' + (h.rule === 'close' ? 'the search' : h.rule)); }
function updateUndo() {
  $('#undo').disabled = !app.hist.length;
  $('#redo').disabled = !app.redo.length;
  const st = state(), open = st && !st.done && st.goals.length;
  $('#closeall').disabled = !open && !(app.verdict && app.verdict.word === 'Unknown');
  $('#hint').disabled = !open;
}
function closeGoal(all) {
  const pf = proof(), st = state(); if (!pf || !st || st.done) return;
  closePop();
  if (all && pf.closeAll[app.sk]) { searchInto(pf.closeAll[app.sk], 'all'); return; }
  const c = pf.close[app.sk + ' ' + app.active] || (all && st.goals.length === 1 && pf.close[app.sk + ' ' + parseGoal(st.goals[0]).n]);
  if (!c) { toast('No scripted answer for this goal in this mockup'); return; }
  if (c.to) { searchInto(c.to, app.active); return; }
  showVerdict(c);
}
function searchInto(to, goal) {
  app.hist.push({ from: app.sk, to, goal: goal === 'all' ? app.active : goal, rule: 'close', by: 'search' });
  app.redo = [];
  go(to, true);
  const msg = goal === 'all' ? 'All goals closed' : 'Goal ' + goal + ' closed';
  toast(msg); announce(msg + '. Proof checked.');
}
// a verdict on the whole sequent goes to the status light; on a subgoal, to the goal
function showVerdict(c) {
  const pf = proof();
  const root = state().goals.length === 1 && !app.hist.length;
  let v;
  if (c.verdict === 'unbalanced') {
    const why = tx(c.extra) + ' occurs once more than ' + c.atom + ', so they cannot all meet in axioms.';
    v = { lamp: 'bad', word: 'Unprovable', tone: 'bad', checked: true, kw: 'atom balance', rows: baseRows(pf, [['Refutation', 'atom balance (' + c.atom + ')']]), reason: fmtText(why),
      extra: '<details><summary>Why the count decides</summary><p>Every axiom uses one ' + mathHTML(c.atom) + ' and one ' + mathHTML('~' + c.atom) + '; no rule here makes or deletes an atom. In ' + mathHTML(c.seq) + ' one ' + mathHTML(c.extra) + ' is left over.</p></details>',
      actions: '<button type="button" class="btn btn-s" data-act="cert">Certificate</button>', full: why };
  } else if (c.verdict === 'classical') {
    const why = 'Read classically, the sequent is false where B is true and A false; every provable sequent reads as a tautology.';
    v = { lamp: 'bad', word: 'Unprovable', tone: 'bad', checked: true, kw: 'countermodel', rows: baseRows(pf, [['Refutation', 'classical countermodel']]), reason: fmtText(why),
      extra: '<details open><summary>The countermodel</summary><table><tr><th>Atom</th>' + c.atoms.map(a => '<td>' + mathHTML(a) + '</td>').join('') + '</tr><tr><th>Value</th>' + c.assignment.map(x => '<td>' + (x ? 'true' : 'false') + '</td>').join('') + '</tr></table><p class="sub">The classical reading erases ! and ?, reads ⊗ and &amp; as and, ⅋ and ⊕ as or, 1 and ⊤ as true, ⊥ and 0 as false.</p></details>',
      actions: '<button type="button" class="btn btn-s" data-act="cert">Certificate</button>', full: why };
  } else if (c.verdict === 'exhausted') {
    const why = 'The search tried every proof and found none. No independent check and no certificate exist for this kind of answer.';
    v = { lamp: 'bad', word: 'Unprovable', tone: 'bad', checked: false, unchecked: true, kw: 'exhaustive search', rows: baseRows(pf, [['Refutation', 'exhaustive search']]), reason: why, full: why };
  } else if (c.verdict === 'unknown') {
    v = unknownV(c);
  }
  if (root) { app.verdict = v; refreshStatus(); openStatus(false); }
  else {
    setDockLine({ kind: 'verdict', html: '<button type="button" class="kw ' + v.tone + '" data-tip="' + esc(v.full) + '" aria-label="' + esc(v.word + (v.checked ? ', checked' : '') + ': ' + v.full) + '">' + esc(v.word) + (v.checked ? ', checked' : '') + ': ' + esc(v.kw) + '</button><button type="button" class="btn btn-s" data-act="undo" data-key="U"><svg class="ic" aria-hidden="true"><use href="#i-undo"/></svg>Undo</button>', wire: d => wireActs(d) });
  }
  updateUndo();
  announce(v.word + (v.checked ? ', checked' : '') + ': ' + (v.full || ''));
}
function unknownV(c) {
  const pf = proof();
  return { lamp: 'warn', word: 'Unknown', tone: 'warn', rows: baseRows(pf, [['Budget', fmtN(c.limit) + ' units'], ['Used', fmtN(103939) + ' in ' + c.after], ['Copy bound', String(c.copies)]]),
    reason: 'No answer within the budget. Not “unprovable”.', full: 'No answer within the budget.',
    extra: '<details><summary>What the search did</summary><table><tr><th>Stable sequents</th><td>4 812</td></tr><tr><th>From the memo</th><td>3 094</td></tr><tr><th>Splits</th><td>10 794</td></tr></table></details>',
    actions: '<button type="button" class="btn warn" data-act="retry" data-key="⇧C">Try ' + fmtN(c.limit * 2) + ' units</button>' };
}
function wireActs(d) {
  $$('[data-act]', d).forEach(b => b.addEventListener('click', () => {
    const a = b.dataset.act;
    if (a === 'undo') undo();
    if (a === 'cert') { closeStatus(); openMenu('export', 'cert'); }
    if (a === 'retry') retry();
    if (a === 'stop') stopSearch();
  }));
}
const fmtN = n => String(n).replace(/\B(?=(\d{3})+(?!\d))/g, ' ');
function retry() {
  const pf = proof();
  app.retry = true;
  app.verdict = { lamp: 'run', word: 'Searching', tone: 'warn', rows: baseRows(pf, [['Budget', fmtN(200000) + ' units'], ['Last answer', 'unknown at ' + fmtN(100000)]]),
    reason: 'Searching with twice the budget; nothing is decided until it answers.', extra: '<div class="spin" role="progressbar" aria-label="Searching"></div>',
    actions: '<button type="button" class="btn btn-s" data-act="stop" data-key="Esc">Stop the search</button>' };
  refreshStatus(); openStatus(false);
  announce('Searching again with a budget of 200000.');
}
function stopSearch() {
  if (!app.retry) return;
  app.retry = false;
  app.verdict = unknownV(proof().close['s0 0']);
  refreshStatus();
  toast('Search stopped; still unknown');
}

// ---------- hints (D12): four rungs at the goal, each one thing, on request
const HINTS = {
  'mll s0': ['Goal 0: an invertible connective first', 'The formula ~A ⅋ ~B', '⅋ on ~A ⅋ ~B', { apply: { to: 's1', goal: 0, pos: 0, rule: '⅋' }, why: 'Why ⅋ first?', reason: '⅋ is invertible: its premise is provable exactly when its conclusion is, so it never leads into a dead end.' }],
  'mll s1': ['Goal 1: its ⊗, a split', 'The formula B ⊗ A', '⊗ on B ⊗ A, with ~B left', { apply: { to: 's2', goal: 1, pos: 2, rule: '⊗', left: [1] }, why: 'Why ~B left?', reason: '~B must meet B in an axiom, and B is the left half of the ⊗; ~A meets A on the right.' }],
};
const HINT_FOCUS = { 'mll s0': [0, 0], 'mll s1': [1, 2] };
function rungsHTML(r) { return '<span class="rungs" aria-hidden="true">' + [1, 2, 3, 4].map(i => '<i class="' + (i <= r ? 'on' : '') + '"></i>').join('') + '</span>'; }
function hint() {
  const pf = proof(), st = state(); if (!pf || !st || st.done) return;
  closePop();
  const key = app.proof + ' ' + app.sk, H = HINTS[key];
  if (!H) { toast('Hints are scripted for A ⊗ B ⊢ B ⊗ A, first two steps'); return; }
  const [fg, fp] = HINT_FOCUS[key];
  if (app.active !== fg) { app.active = fg; markActive(); focusGoal(true); }
  app.hint = Math.min(4, app.hint + 1); updateHintRung();
  const r = app.hint;
  $$('.fb.hinted').forEach(b => b.classList.remove('hinted'));
  if (r === 2 || r === 3) { const b = hits.querySelector('.fb[data-goal="' + fg + '"][data-pos="' + fp + '"]'); if (b) b.classList.add('hinted'); }
  if (r < 4) {
    setDockLine({ kind: 'hint', html: '<span class="kw acc">Hint ' + r + ' of 4 ' + rungsHTML(r) + '</span><span class="txt">' + fmtText(H[r - 1]) + '</span><button type="button" class="btn btn-s" data-h="more" data-key="H">' + (r === 3 ? 'Apply it' : 'More') + '</button>',
      wire: d => { $('[data-h="more"]', d).addEventListener('click', hint); } });
    announce('Hint ' + r + ' of 4: ' + H[r - 1]);
  } else {
    const B = H[3];
    app.hint = 0; updateHintRung();
    step({ to: B.apply.to, goal: B.apply.goal, pos: B.apply.pos, rule: B.apply.rule, left: B.apply.left, by: 'hint' }, true);
    setDockLine({ kind: 'hint', html: '<span class="kw acc">Hint 4 of 4 ' + rungsHTML(4) + '</span><span class="txt">' + ruleHTML(B.apply.rule) + ' applied. ' + fmtText(B.why) + '</span><button type="button" class="btn btn-s" data-h="why" aria-expanded="false">Answer</button><div class="why" hidden style="flex-basis:100%"><label class="sr-only" for="why-in">Your reason</label><textarea id="why-in" placeholder="Your reason"></textarea> <button type="button" class="kw acc" data-tip="' + esc(tx(B.reason)) + '">Compare</button></div>',
      wire: d => { const b = $('[data-h="why"]', d); b.addEventListener('click', () => { const w = $('.why', d); w.hidden = !w.hidden; b.setAttribute('aria-expanded', String(!w.hidden)); if (!w.hidden) $('#why-in').focus(); layoutDock(); }); } });
    announce('The page applied ' + B.apply.rule + '. ' + B.why);
  }
}
function fmtText(s) {
  // formulas and connectives inside a sentence, set as linlog draws them
  return esc(s).replace(/(?<![A-Za-z])([?!]?~?[A-Z](?: [⊗⅋⊸] [?!]?~?[A-Z])?)(?![A-Za-z'])|([⊗⅋⊸⊢⊕⊤⊥]|&amp;L[₁₂]|&amp;R|1L|[A-Z]⊥)/g, (m, f, c) => f ? mathHTML(f) : '<span class="m">' + c + '</span>');
}
function updateHintRung() { const r = $('#hint-rung'); r.hidden = !app.hint; r.textContent = app.hint + '/4'; }

// ---------- view (D4)
function renderViewTabs() {
  $('.stage-ctl').hidden = !proof();
  const pf = proof(), st = state();
  const tree = $('#tab-tree'), net = $('#tab-net');
  tree.setAttribute('aria-selected', String(app.view === 'tree'));
  net.setAttribute('aria-selected', String(app.view === 'net'));
  let reason = '';
  if (!pf) reason = 'Start a proof first.';
  else if (!pf.net) reason = pf.netWhy;
  else if (!st.done) reason = 'The net appears once the proof is finished.';
  net.setAttribute('aria-disabled', String(!!reason));
  if (reason) net.dataset.tip = reason; else delete net.dataset.tip;
}
function setView(v) {
  const pf = proof(), st = state();
  if (v === 'net' && (!pf || !pf.net || !st.done)) { showTip($('#tab-net')); announce($('#tab-net').dataset.tip || ''); return; }
  if (v === app.view) return;
  app.view = v;
  closePop();
  drawState(false); fit(false); renderViewTabs(); renderDock();
}

// ---------- the top band is one row (D53): each tab is a proof; several at once (D50)
function snapshot() {
  return { key: app.proof, sk: app.sk, hist: app.hist, redo: app.redo, active: app.active, verdict: app.verdict, view: app.view, mode: app.mode, mix: app.mix, affine: app.affine, hint: app.hint };
}
function saveTab() { if (app.cur >= 0 && app.tabs[app.cur] !== undefined && app.proof) app.tabs[app.cur] = snapshot(); }
function restoreTab(i) {
  const t = app.tabs[i]; app.cur = i;
  Object.assign(app, { proof: t.key, hist: t.hist, redo: t.redo, active: t.active, view: t.view, mode: t.mode, mix: t.mix, affine: t.affine, hint: t.hint, prevDesc: null, editing: null });
  const v = t.verdict;
  setPart(P[t.key].ordinary ? 'ordinary' : 'linear', true); syncModeUI();
  setFieldMsg('');
  go(t.sk, false);
  if (v) { app.verdict = v; refreshStatus(); }
  updateHintRung();
}
function switchTab(i) {
  if (i === app.cur) return;
  closePop(); closeStatus(); closeMenu(); closeCalc();
  saveTab(); restoreTab(i);
  announce('Switched to ' + P[app.proof].uni + '.');
}
function closeTab(i) {
  saveTab();
  const name = P[app.tabs[i].key].uni;
  app.tabs.splice(i, 1);
  if (!app.tabs.length) { app.cur = -1; app.proof = null; app.editing = 'new'; clearStage(); renderProofRow(); announce(name + ' closed.'); return; }
  app.cur = -1; restoreTab(Math.min(i, app.tabs.length - 1));
  announce(name + ' closed.');
}
function tabTone(t) {
  const st = P[t.key].states[t.sk];
  if (t.verdict) return t.verdict.lamp === 'run' ? 'run' : t.verdict.lamp;
  return st.done ? 'ok' : 'open';
}
// the calculus as a mark: LL, ILL, LL+mix, LL+affine; ordinary logic as LK
function markOf(t) {
  if (P[t.key].ordinary) return 'LK';
  return (t.mode === 'intuitionistic' ? 'ILL' : 'LL') + (t.mix ? '+mix' : '') + (t.affine ? '+affine' : '');
}
function calcName(t, pf) {
  if (!t) return 'ordinary classical logic';
  return (t.mode === 'intuitionistic' ? 'intuitionistic' : 'classical') + ' linear logic' + (t.mix ? ' with Mix' : '') + (t.affine ? ', affine' : '');
}
function renderProofRow() {
  if (app.cur >= 0 && app.proof) app.tabs[app.cur] = snapshot();
  const row = $('#proofs'), form = $('#seq-form');
  document.body.appendChild(form);
  row.innerHTML = app.tabs.map((t, i) => {
    const cur = i === app.cur, v = cur ? verdictNow() : null, tone = tabTone(t);
    const word = v ? v.word + (v.checked ? ', checked' : '') : { ok: 'Proved', open: 'Open', bad: 'Unprovable', warn: 'Unknown', run: 'Searching' }[tone];
    const editing = cur && app.editing === 'tab';
    return '<span class="ptab' + (editing ? ' editing' : '') + '" aria-current="' + cur + '">' +
      '<button type="button" class="pt-dot" data-dot="' + i + '" aria-haspopup="dialog" aria-expanded="false" aria-label="Verdict: ' + esc(word) + '. Details" data-tip="' + esc(word) + '"><span class="dot ' + tone + '" aria-hidden="true"></span></button>' +
      (editing ? '<span class="slot"></span>' : '<button type="button" class="pt-go" data-t="' + i + '" aria-label="' + esc(P[t.key].uni + (cur ? ', click to edit' : '')) + '" data-tip="' + (cur ? 'Edit the sequent' : 'Switch to this proof') + '">' + mathHTML(P[t.key].uni) + '</button>') +
      '<button type="button" class="pt-mark" data-mk="' + i + '" aria-haspopup="dialog" aria-expanded="false" aria-label="Calculus: ' + esc(calcName(P[t.key].ordinary ? null : t)) + '" data-tip="' + esc('Calculus: ' + calcName(P[t.key].ordinary ? null : t)) + '">' + markOf(t) + '</button>' +
      '<button type="button" class="pt-x" data-x="' + i + '" aria-label="Close this proof" data-tip="Close this proof"><svg class="ic" aria-hidden="true"><use href="#i-x"/></svg></button></span>';
  }).join('') +
    (app.editing === 'new' ? '<span class="ptab editing" aria-current="true"><span class="slot"></span></span>' : '') +
    '<button type="button" class="pt-new" id="pt-new" aria-label="New proof" data-tip="New proof"><svg class="ic" aria-hidden="true"><use href="#i-plus"/></svg></button>';
  const slot = $('.slot', row);
  if (slot) { slot.replaceWith(form); form.hidden = false; } else form.hidden = true;
  $$('[data-t]', row).forEach(b => b.addEventListener('click', () => { const i = +b.dataset.t; if (i === app.cur) editTab(); else switchTab(i); }));
  $$('[data-x]', row).forEach(b => b.addEventListener('click', () => closeTab(+b.dataset.x)));
  $$('[data-mk]', row).forEach(b => b.addEventListener('click', () => { const i = +b.dataset.mk; if (i !== app.cur) switchTab(i); openCalc(); }));
  $$('[data-dot]', row).forEach(b => {
    b.addEventListener('click', () => { const i = +b.dataset.dot; if (i !== app.cur) switchTab(i); if (statusPinned) closeStatus(); else openStatus(true); });
    b.addEventListener('mouseenter', () => { if (+b.dataset.dot === app.cur && $('#status-pop').hidden) openStatus(false); });
    b.addEventListener('mouseleave', () => setTimeout(() => { if (!statusPinned && !$('#status-pop').matches(':hover')) closeStatus(); }, 150));
  });
  $('#pt-new').addEventListener('click', newProof);
  const curTab = $('.ptab[aria-current="true"]', row);
  if (curTab) { // keep the active tab in view by scrolling the row only
    const rr = row.getBoundingClientRect(), tr = curTab.getBoundingClientRect();
    if (tr.right > rr.right) row.scrollLeft += tr.right - rr.right + 8; else if (tr.left < rr.left) row.scrollLeft -= rr.left - tr.left + 8;
  }
  placeFieldMsg();
}
// the sequent field: in place of the tab's text, or in a new tab (D53)
function editTab() {
  closePop(); closeStatus(); closeCalc();
  app.editing = 'tab';
  renderProofRow();
  const f = $('#field'); f.value = proof().input; f.focus(); f.select();
}
function newProof() {
  closePop(); closeStatus(); closeCalc();
  app.editing = 'new';
  renderProofRow();
  const f = $('#field'); f.value = ''; f.focus();
  if (!app.tabs.length) emptyLine();
}
function cancelEdit() {
  if (!app.editing || !app.tabs.length) return;
  app.editing = null; setFieldMsg(''); renderProofRow();
  const b = $('#proofs .ptab[aria-current="true"] .pt-go'); if (b) b.focus();
}
function emptyLine() {
  setFieldMsg('<span>Type a sequent, or open one from <button type="button" class="link" id="go-practice">Practice</button>.</span>');
  $('#go-practice').addEventListener('click', () => { toggleLearn(true); setRubric('practice'); });
}

// ---------- loading a sequent: a new tab, the existing one, or the tab being edited
function load(key, opts) {
  opts = opts || {};
  closePop(); closeStatus(); closeCalc();
  const replace = app.editing === 'tab' ? app.cur : opts.replace;
  saveTab();
  const pf = P[key];
  Object.assign(app, { proof: key, hist: [], redo: [], view: 'tree', hint: 0, prevDesc: null, verdict: null, retry: false, active: null, reason: null, editing: null });
  if (!pf.ordinary) app.mode = pf.mode;
  const existing = app.tabs.findIndex((t, i) => t && t.key === key && i !== replace);
  if (replace !== undefined && replace !== null && replace >= 0) { if (existing >= 0) { app.tabs.splice(replace, 1); app.cur = existing > replace ? existing - 1 : existing; } else app.cur = replace; }
  else if (existing >= 0) app.cur = existing;
  else { app.tabs.push(null); app.cur = app.tabs.length - 1; }
  setPart(pf.ordinary ? 'ordinary' : 'linear', true);
  syncModeUI();
  setFieldMsg('');
  updateHintRung();
  go(opts.sk || 's0', false);
  if (!opts.quiet) announce('Opened ' + pf.uni + (pf.ordinary ? '' : ', ' + app.mode) + '.');
}
function setFieldMsg(html, invalid) {
  const m = $('#field-msg');
  m.innerHTML = html; m.hidden = !html;
  $('#field').setAttribute('aria-invalid', invalid ? 'true' : 'false');
  if (!invalid) $('#caret-mark').hidden = true;
  placeFieldMsg();
}
function placeFieldMsg() {
  const m = $('#field-msg'), form = $('#seq-form');
  if (m.hidden || form.hidden) { m.style.visibility = 'hidden'; return; }
  m.style.visibility = '';
  const r = form.getBoundingClientRect();
  m.style.top = (r.bottom + 8) + 'px';
  m.style.left = Math.max(8, Math.min(r.left, innerWidth - m.offsetWidth - 8)) + 'px';
}
const PARSE = { 'A ** B |- A': 'A____B____A', 'A |- B |- C': 'A____B____C', 'A * |- B': 'A______B', 'A -o (B |- C': 'A__o__B____C', 'foo bar |- baz': 'foo_bar____baz' };
function norm(s) { return s.trim().replace(/\s+/g, ' '); }
function submitField() {
  const s = norm($('#field').value);
  if (!s) { $('#field').value = $('#field').placeholder; return submitField(); }
  if (app.mode === 'intuitionistic' && app.part === 'linear' && s === 'A |- B, C') return parseError(s, null, 'ill-two-right');
  if (PARSE[s]) return parseError(s, D['parse/' + PARSE[s] + '.txt.stderr']);
  const k = Object.keys(P).find(k => norm(P[k].input) === s || norm(P[k].uni) === s);
  if (!k) { setFieldMsg('<span class="kw plain" tabindex="0" data-tip="Only the examples have scripted states here; Practice lists them.">Not scripted in this mockup</span>'); return; }
  let key = k;
  if (P[k].ill && app.mode === 'intuitionistic') key = P[k].ill;
  load(key);
}
// a parse error: a mark under the place, inside the field, and keywords below it (D15)
function markColumn(col, len) {
  const f = $('#field'), cm = $('#caret-mark');
  const cs = getComputedStyle(f);
  const ctx = document.createElement('canvas').getContext('2d');
  ctx.font = cs.font;
  const x0 = parseFloat(cs.paddingLeft) + parseFloat(cs.borderLeftWidth) + ctx.measureText(f.value.slice(0, col)).width - f.scrollLeft;
  const w = Math.max(6, ctx.measureText(f.value.slice(col, col + (len || 1))).width);
  Object.assign(cm.style, { left: (f.offsetLeft + x0) + 'px', width: w + 'px' });
  cm.hidden = false;
}
function parseError(input, stderr, special) {
  // the stage stays as it is
  if (special === 'ill-two-right') {
    const at = $('#field').value.indexOf('|-');
    const full = 'In intuitionistic mode the right of ⊢ holds exactly one formula, not 2: write the hypotheses left of ⊢.';
    setFieldMsg('<button type="button" class="kw bad" data-tip="' + esc(full) + '" aria-label="' + esc(full) + '">2 formulas right of ⊢</button>', true);
    markColumn(at, $('#field').value.length - at);
    announce(full);
    return;
  }
  const lines = stderr.split('\n');
  const cl = lines[2];
  const col = cl.indexOf('^') - 2;
  const m = cl.match(/\^ unexpected "(.*?)", expected (.*)$/);
  const want = { 'a formula': 'a formula', 'a connective, `,` or `|-`': 'a connective, a comma or |-' }[m[2]] || m[2];
  const full = 'Unexpected ' + m[1] + ' at column ' + (col + 1) + '; expected ' + want + '. The stage keeps what is on it.';
  setFieldMsg('<button type="button" class="kw bad" data-tip="' + esc(full) + '" aria-label="' + esc(full) + '">Unexpected ' + esc(m[1]) + ' at column ' + (col + 1) + '</button><span class="kw plain">Expected ' + esc(want) + '</span>', true);
  const f = $('#field'); f.value = input; f.focus(); try { f.setSelectionRange(col, col + 1); } catch (e) {}
  markColumn(col, 1);
  announce(full);
}

// ---------- the calculus popover of a tab (D53); a change restarts the proof, and says so first (D2)
function openCalc() {
  const b = $('#proofs .ptab[aria-current="true"] .pt-mark'); if (!b) return;
  closeStatus(); closeMenu(); closePop();
  const c = $('#calc-pop');
  syncModeUI(); $('#mode-warn').hidden = true;
  c.hidden = false; b.setAttribute('aria-expanded', 'true');
  const r = b.getBoundingClientRect();
  c.style.top = (r.bottom + 8) + 'px';
  c.style.left = Math.max(8, Math.min(r.left - 20, innerWidth - c.offsetWidth - 8)) + 'px';
  const first = $('[aria-checked="true"]', c); if (first) first.focus();
}
function closeCalc() { $('#calc-pop').hidden = true; $$('.pt-mark').forEach(b => b.setAttribute('aria-expanded', 'false')); }
function setPart(part, silent) {
  app.part = part;
  $$('[data-part]').forEach(b => b.setAttribute('aria-checked', String(b.dataset.part === part)));
  $('#calc-seg').hidden = part !== 'linear'; $('#sw-row').hidden = part !== 'linear';
  $('#ord-seg').hidden = part !== 'ordinary';
  if (!silent && part === 'ordinary') { closeCalc(); load('ordinary'); }
}
function syncModeUI() {
  $$('[data-mode]').forEach(b => b.setAttribute('aria-checked', String(b.dataset.mode === app.mode)));
  $('#mix').checked = app.mix; $('#affine').checked = app.affine;
}
let pendingMode = null;
function requestMode(change) {
  const pf = proof(), steps = app.hist.length;
  if (!pf || !steps) { applyMode(change); return; }
  pendingMode = change;
  const what = change.mode ? change.mode : (change.mix !== undefined ? (change.mix ? 'with Mix' : 'without Mix') : (change.affine ? 'affine' : 'not affine'));
  $('#mode-warn-t').innerHTML = '<span class="kw warn" tabindex="0" data-tip="' + esc('A session is one sequent in one calculus: switching restarts ' + pf.uni + ' and drops its ' + steps + ' step' + (steps > 1 ? 's' : '') + '. Export the session first to keep them.') + '">Restarts the proof, drops ' + steps + ' step' + (steps > 1 ? 's' : '') + '</span>';
  $('#mode-go').textContent = 'Restart ' + what;
  $('#mode-warn').hidden = false;
  $('#mode-go').focus();
  syncModeUI();
}
function applyMode(change) {
  $('#mode-warn').hidden = true;
  Object.assign(app, change);
  syncModeUI();
  const pf = proof(); if (!pf) return;
  let key = app.proof;
  if (change.mode === 'intuitionistic' && pf.ill) key = pf.ill;
  if (change.mode === 'classical' && key === 'illfirst') key = 'first';
  if (change.mode && P[key].mode !== app.mode) { toast('No ' + app.mode + ' session for ' + pf.uni + ' in this mockup'); return; }
  const keep = { mix: app.mix, affine: app.affine };
  load(key, { quiet: true, replace: app.cur });
  Object.assign(app, keep); syncModeUI(); renderProofRow();
  closeCalc();
  toast('Restarted in ' + calcName({ mode: app.mode, mix: app.mix, affine: app.affine }));
}
function clearStage() {
  drawing.innerHTML = ''; hits.innerHTML = ''; geo = null; dockEl.hidden = true;
  renderViewTabs(); updateUndo(); updateMinimap();
}

// ---------- the bottom band's panels: Export, Import, Settings, Steps (one level)
const menuEl = $('#menu');
function openMenu(kind, focusId) {
  closePop(); closeStatus();
  if (app.menu === kind && !menuEl.hidden && !focusId) { closeMenu(); return; }
  app.menu = kind;
  renderMenu(kind, focusId);
}
function closeMenu() {
  menuEl.hidden = true; app.menu = null;
  $$('#export-b,#import-b,#settings-b,#steps-b,#about-b').forEach(b => b.setAttribute('aria-expanded', 'false'));
}
function exportsList() {
  const pf = proof(); if (!pf) return [];
  const st = state(), e = pf.exports, done = st.done;
  const L = [];
  const add = (id, fmt, file, tip, kind) => L.push({ id, fmt, file, tip, kind });
  const svg = done || !st.svg ? e.svg : st.svg;
  if (svg) add('svg', 'SVG', svg, done ? 'The derivation in Euler Math' : 'The derivation so far, open goals as bare sequents');
  if (e.svg || st.svg) { add('png', 'PNG', e.png, 'Rendered from the SVG with the page\'s fonts'); add('pdf', 'PDF', e.pdf, 'Rendered from the SVG with the page\'s fonts'); }
  if (e.tex) add('tex', 'LaTeX', e.tex, 'ebproof');
  if (e.typ) add('typ', 'Typst', e.typ, 'curryst');
  if (e.txt) add('txt', 'Text', e.txt, 'The verdict and the tree as text');
  if (st.json) add('session', 'Session', st.json, 'The session document, JSON: what a link carries and Import reads');
  if (e.json) add('json', 'JSON', e.json, done ? 'The checked proof term' : 'The outcome');
  if (e.v) add('v', 'Rocq', e.v, 'Certificate of the proof (NanoYalla)');
  if (e.cert) add('cert', 'Rocq', e.cert, 'Certificate of non-derivability');
  if (e.nocert) add('nocert', 'Rocq', null, 'No certificate: an exhausted search has no check');
  if (e.netsvg && done) add('netsvg', 'Net SVG', e.netsvg, 'The proof net');
  if (e.netpng && done) add('netpng', 'Net PNG', e.netpng, 'The proof net');
  return L;
}
async function renderMenu(kind, focusId) {
  const anchor = { export: '#export-b', import: '#import-b', settings: '#settings-b', steps: '#steps-b', about: '#about-b' }[kind];
  $$('#export-b,#import-b,#settings-b,#steps-b,#about-b').forEach(b => b.setAttribute('aria-expanded', String(b.matches(anchor))));
  let h = '';
  if (kind === 'export') {
    const L = exportsList();
    if (!L.length) h = '<p class="sub" style="margin:6px">Start a proof to export it.</p>';
    else {
      h = '<ul class="mlist" role="menu" aria-label="Export">' + L.map(x => {
        const fn = x.file ? x.file.split('/').pop() : '';
        if (!x.file) return '<li role="none"><span class="mi" role="menuitem" aria-disabled="true" tabindex="0" data-tip="' + esc(x.tip + (x.id === 'png' || x.id === 'pdf' ? ' (no file in this mockup\'s data for this sequent)' : '')) + '"><span class="fmt">' + esc(x.fmt) + '</span><span class="fn">' + (x.id === 'nocert' ? 'None' : 'On request') + '</span></span></li>';
        return '<li role="none"><a class="mi" role="menuitem" data-id="' + x.id + '" href="data/' + esc(x.file) + '" download data-tip="' + esc(x.tip) + '"><span class="fmt">' + esc(x.fmt) + '</span><span class="fn">' + esc(fn) + '</span></a></li>';
      }).join('') + '<li class="msep" role="separator"></li><li role="none"><button type="button" class="mi" role="menuitem" id="copy-l" data-id="link"><span class="fmt">Link</span><span class="fn">Copy link</span><span class="lk" id="lk">…</span></button></li></ul>';
    }
  } else if (kind === 'import') {
    h = '<div class="form" aria-label="Import"><span class="kw-label h">Import a session or a sequent</span>' +
      '<div class="row"><button type="button" class="btn btn-primary" id="imp-file"><svg class="ic" aria-hidden="true"><use href="#i-import"/></svg>Choose a .json file</button><span class="sub">or drop one on the stage</span></div>' +
      '<label class="sr-only" for="imp-text">Paste JSON</label><textarea id="imp-text" placeholder="Paste JSON: a session (save) or a sequent (seq json)"></textarea>' +
      '<div class="row"><button type="button" class="btn" id="imp-go">Import</button><span id="imp-res"></span></div>' +
      '<div class="row sub">Samples for the reviewer: <button type="button" class="btn btn-s" data-sample="session">Session</button><button type="button" class="btn btn-s" data-sample="sequent">Sequent</button><button type="button" class="btn btn-s" data-sample="level">Level 9</button></div></div>';
  } else if (kind === 'settings') {
    const th = document.documentElement.getAttribute('data-theme') || 'system';
    h = '<div class="form" aria-label="Settings"><span class="kw-label h">Settings</span>' +
      '<div class="lbl"><span>Theme</span><div class="seg" role="radiogroup" aria-label="Theme">' + ['system', 'light', 'dark'].map(t => '<button type="button" role="radio" class="seg-b" data-th="' + t + '" aria-checked="' + (th === t) + '">' + t.charAt(0).toUpperCase() + t.slice(1) + '</button>').join('') + '</div></div>' +
      '<div class="lbl"><span>Budget</span><span class="row"><input class="num-in" id="budget" value="' + app.budget + '" inputmode="numeric" aria-label="Search budget, units of work"><span class="sub" data-tip="Work, not time: the library reads no clock in the browser.">units of work</span></span></div>' +
      '<div class="lbl"><span>Labels</span><span class="row"><label class="sw"><input type="checkbox" id="set-lbl"' + (app.labels ? ' checked' : '') + '><span class="sw-t" aria-hidden="true"></span>Rule names</label><label class="sw"><input type="checkbox" id="set-search"' + (app.searchLight ? ' checked' : '') + '><span class="sw-t" aria-hidden="true"></span>Search\'s steps lighter</label></span></div>' +
      '<div class="lbl"><span>Keys</span><span class="row"><label class="sw"><input type="checkbox" id="set-keys"' + (document.documentElement.classList.contains('show-keys') ? ' checked' : '') + '><span class="sw-t" aria-hidden="true"></span>Show keys beside actions</label></span></div>' +
      '<div class="lbl"><span>Offline</span><span>Installable; works offline after one visit (install from the browser\'s menu)</span></div>' +
      '</div>';
  } else if (kind === 'about') {
    h = '<div class="form about" aria-label="About"><span class="kw-label h">About</span>' +
      '<p>linlog\'s prover, running in your browser. Nothing you type is sent anywhere.</p>' +
      '<div class="lbl"><span>Library</span><span>linlog 0.1.0</span></div>' +
      '<div class="lbl"><span>Source</span><a href="https://github.com/linlog-prover/linlog">github.com/linlog-prover/linlog</a></div>' +
      '<div class="lbl"><span>Documentation</span><a href="https://docs.rs/linlog">docs.rs/linlog</a></div>' +
      '<div class="lbl"><span>Licences</span><ul class="lic"><li>Code: EUPL-1.2</li><li>Written content: CC BY 4.0</li><li>Fonts: SIL Open Font License</li><li><a href="third-party-notices.txt">Third-party notices</a></li></ul></div>' +
      '<div class="lbl"><span>Feedback</span><a href="https://github.com/linlog-prover/linlog/issues">Issues on GitHub</a></div>' +
      '<div class="lbl"><span>Privacy</span><span class="sub">No tracking. GitHub logs visitors\' IP addresses for every Pages site.</span></div></div>';
  } else if (kind === 'steps') {
    const all = app.hist.concat(app.redo.slice().reverse());
    h = '<span class="kw-label h">Steps</span>' + (proof() ? '<ol class="steps"><li><button type="button" data-s="-1" aria-current="' + (app.hist.length === 0) + '"><span class="n">0</span><span class="r">Start</span><span>' + mathHTML(proof().uni) + '</span><span class="by"></span></button></li>' + all.map((s, i) =>
      '<li><button type="button" data-s="' + i + '" class="' + (i >= app.hist.length ? 'later' : '') + '" aria-current="' + (i === app.hist.length - 1) + '"><span class="n">' + (i + 1) + '</span><span class="r">' + esc(s.rule === 'close' ? 'search' : s.rule) + '</span><span>Goal ' + s.goal + (s.left ? ', left: ' + s.left.map(p => 'formula ' + p).join(', ') : '') + '</span><span class="by">' + esc({ you: 'You', search: 'Search', hint: 'Hint', lesson: 'Lesson' }[s.by] || s.by) + '</span></button></li>').join('') + '</ol>' : '<p class="sub">No proof open</p>');
  }
  menuEl.innerHTML = h;
  menuEl.hidden = false;
  const a = $(anchor).getBoundingClientRect();
  menuEl.style.left = Math.max(8, Math.min(a.left, innerWidth - menuEl.offsetWidth - 8)) + 'px';
  menuEl.style.top = Math.max(8, a.top - menuEl.offsetHeight - 8) + 'px';
  wireMenu(kind, focusId);
}
function wireMenu(kind, focusId) {
  if (kind === 'export') {
    const c = $('#copy-l');
    if (c) { linkFor().then(l => { const lk = $('#lk'); if (lk) lk.textContent = l.shown; }); c.addEventListener('click', () => linkFor().then(l => copy(l.encoded, 'Link copied'))); }
    const f = focusId ? $('[data-id="' + focusId + '"]', menuEl) : $('.mi', menuEl);
    if (f) { f.focus(); }
  } else if (kind === 'import') {
    $('#imp-file').addEventListener('click', () => $('#file-in').click());
    $('#imp-go').addEventListener('click', () => importText($('#imp-text').value, 'paste'));
    $$('[data-sample]', menuEl).forEach(b => b.addEventListener('click', () => { $('#imp-text').value = SAMPLES[b.dataset.sample](); }));
    $('#imp-text').focus();
  } else if (kind === 'settings') {
    $$('[data-th]', menuEl).forEach(b => b.addEventListener('click', () => { setTheme(b.dataset.th); renderMenu('settings'); }));
    $('#set-lbl').addEventListener('change', e => { app.labels = e.target.checked; stage.classList.toggle('no-labels', !app.labels); });
    $('#set-search').addEventListener('change', e => { app.searchLight = e.target.checked; stage.classList.toggle('search-plain', !app.searchLight); });
    $('#budget').addEventListener('change', e => { app.budget = +e.target.value || app.budget; });
    $('#set-keys').addEventListener('change', e => setShowKeys(e.target.checked));
  } else if (kind === 'steps') {
    $$('[data-s]', menuEl).forEach(b => b.addEventListener('click', () => gotoStep(+b.dataset.s)));
  }
}
function gotoStep(i) {
  const all = app.hist.concat(app.redo.slice().reverse());
  app.hist = all.slice(0, i + 1);
  app.redo = all.slice(i + 1).reverse();
  go(i < 0 ? 's0' : all[i].to, true);
}
async function deflateB64(text) {
  try {
    const cs = new CompressionStream('deflate-raw');
    const buf = await new Response(new Blob([text]).stream().pipeThrough(cs)).arrayBuffer();
    let bin = ''; new Uint8Array(buf).forEach(b => bin += String.fromCharCode(b));
    return btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  } catch (e) { return null; }
}
async function linkFor() {
  const pf = proof(), st = state();
  const mode = pf.ordinary ? 'classical-logic' : app.mode + (app.mix ? '+mix' : '') + (app.affine ? '+affine' : '');
  const base = 'https://linlog-prover.github.io/';
  const b64 = st.json ? await deflateB64(D[st.json].trim()) : null;
  return { shown: base + '#v1/' + pf.uni + '/' + mode + (b64 ? '/' + b64 : ''), encoded: base + '#v1/' + encodeURIComponent(pf.uni) + '/' + mode + (b64 ? '/' + b64 : '') };
}
function copy(text, msg) {
  try { navigator.clipboard.writeText(text).then(() => toast(msg), () => fallbackCopy(text, msg)); } catch (e) { fallbackCopy(text, msg); }
}
function fallbackCopy(text, msg) {
  const t = document.createElement('textarea'); t.value = text; document.body.appendChild(t); t.select();
  try { document.execCommand('copy'); toast(msg); } catch (e) { toast('Select the link and copy it'); }
  t.remove();
}

// ---------- import (D51): a session (save) or a sequent (seq json); the page names which
const SAMPLES = {
  session: () => D['mll/interact/state-2.json'].trim(),
  sequent: () => { const s = JSON.parse(D['mll/interact/state-0.json']).sequent; return JSON.stringify(Object.assign({ version: 1 }, s)); },
  level: () => D['mll/interact/state-2.json'].trim().replace('"version":1', '"version":9'),
};
const canon = o => JSON.stringify(o);
function importText(text, via) {
  let o;
  const res = (tone, kw, tip) => { const r = $('#imp-res'); const h = '<span class="kw ' + tone + '" tabindex="0"' + (tip ? ' data-tip="' + esc(tip) + '"' : '') + '>' + kw + '</span>'; if (r) r.innerHTML = h; return { tone, kw, tip }; };
  try { o = JSON.parse(text); } catch (e) { return res('bad', 'Not JSON', 'The text is not JSON.'); }
  const isSession = o && o.sequent && o.inferences, isSequent = o && o.terms && o.roots;
  const what = isSession ? 'session' : isSequent ? 'sequent' : null;
  if (!what) return res('bad', 'Neither a session nor a sequent', 'Neither a session (save) nor a sequent (seq json).');
  if (![1, 2].includes(o.version)) return res('bad', 'Level ' + o.version + ' unknown: not imported', 'A ' + what + ' at level ' + o.version + '. This page reads levels 1 and 2; a later linlog wrote this.');
  for (const key of Object.keys(P)) {
    for (const sk of Object.keys(P[key].states)) {
      const st = P[key].states[sk];
      if (!st.json) continue;
      const doc = JSON.parse(D[st.json]);
      if (isSession && canon(doc) === canon(o)) { load(key, { sk, quiet: true }); toast('Imported a session: ' + P[key].uni); announce('Imported a session: ' + P[key].uni + '.'); return res('ok', 'Imported a session, level ' + o.version); }
      if (isSequent && sk === 's0' && canon(doc.sequent) === canon({ terms: o.terms, roots: o.roots, atoms: o.atoms, antecedents: o.antecedents })) { load(key, { quiet: true }); toast('Imported a sequent: ' + P[key].uni); announce('Imported a sequent: ' + P[key].uni + '.'); return res('ok', 'Imported a sequent, level ' + o.version); }
    }
  }
  return res('plain', 'A ' + what + ', level ' + o.version, 'Read as a ' + what + ', but the mockup has no scripted states for it.');
}
$('#file-in').addEventListener('change', e => { const f = e.target.files[0]; if (f) f.text().then(t => importText(t, 'file')); });
(function () {
  const drop = $('#drop'); let depth = 0;
  stage.addEventListener('dragenter', e => { e.preventDefault(); depth++; drop.hidden = false; drop.classList.remove('got'); });
  stage.addEventListener('dragover', e => e.preventDefault());
  stage.addEventListener('dragleave', () => { if (--depth <= 0) { depth = 0; drop.hidden = true; } });
  stage.addEventListener('drop', e => {
    e.preventDefault(); depth = 0;
    const f = e.dataTransfer.files[0];
    if (!f) { drop.hidden = true; return; }
    f.text().then(t => { const r = importText(t, 'drop'); showDropResult(r); });
  });
})();
function showDropResult(r) {
  const drop = $('#drop');
  drop.hidden = false; drop.classList.toggle('got', r.tone === 'ok');
  drop.querySelector('.drop-in').innerHTML = '<span class="kw ' + r.tone + '">' + r.kw + '</span>' + (r.tip ? '<p>' + esc(r.tip) + '</p>' : '');
  setTimeout(() => { drop.hidden = true; drop.querySelector('.drop-in').innerHTML = '<svg class="ic big" aria-hidden="true"><use href="#i-import"/></svg><p>Drop a session or a sequent (.json)</p>'; }, DEBUG ? 60000 : 1800);
}

// ---------- theme (D46): the toggle in the band; system in Settings
function currentDark() { const t = document.documentElement.getAttribute('data-theme'); return t ? t === 'dark' : matchMedia('(prefers-color-scheme: dark)').matches; }
function setTheme(t) {
  if (t === 'system') document.documentElement.removeAttribute('data-theme'); else document.documentElement.setAttribute('data-theme', t);
  try { if (t === 'system') localStorage.removeItem('linlog-theme'); else localStorage.setItem('linlog-theme', t); } catch (e) {}
  const b = $('#theme-b'), next = currentDark() ? 'Light theme' : 'Dark theme';
  b.setAttribute('aria-label', next); b.dataset.tip = next;
}
function setShowKeys(on) {
  document.documentElement.classList.toggle('show-keys', on);
  try { localStorage.setItem('linlog-keys', on ? '1' : '0'); } catch (e) {}
}

// ---------- Learn (D47): a frame around the window with three rubrics
function toggleLearn(on) {
  app.learn = on === undefined ? !app.learn : on;
  $('#learn').setAttribute('aria-pressed', String(app.learn));
  $('#app').classList.toggle('learn-on', app.learn);
  $('#frame').hidden = !app.learn;
  if (!app.learn) { app.reason = null; app.lesson = null; app.worked = null; layoutReasons(); }
  else renderFrame();
  setTimeout(() => { if (geo) (state() && !state().done ? focusGoal(false) : fit(false)); placeFieldMsg(); }, reducedMQ.matches ? 0 : 60);
  if (geo && app.view === 'tree') { buildHits(); markActive(); }
}
function setRubric(r) { app.rubric = r; renderFrame(); }

// the transcripts' paths, with a reason and a prediction for each inference (D21, D56)
const PATHS = {
  first: { path: [['s1', 0, 1, '⊗', [0]], ['s2', 1, 0, 'ax'], ['s3', 2, 0, 'ax']], reasons: [
    { rule: '⊗', opts: ['⊗', 'ax', '⅋'], why: 'Only ⊗ has a connective. ~A goes left to meet A; B stays right with ~B.' },
    { rule: 'ax', opts: ['ax', '⊗', '?d'], why: '~A and A alone: an axiom.' },
    { rule: 'ax', opts: ['⅋', 'ax', '⊗'], why: '~B and B alone: an axiom.' }] },
  mll: { path: [['s1', 0, 0, '⅋'], ['s2', 1, 2, '⊗', [1]], ['s3', 2, 0, 'ax'], ['s4', 3, 0, 'ax']], reasons: [
    { rule: '⅋', opts: ['⊗', '⅋', 'ax'], why: '⅋ is invertible: it never loses a proof, so it goes first.' },
    { rule: '⊗', opts: ['⊗', 'ax', '⅋'], why: '~B goes left to meet B, the left half of the ⊗; ~A meets A on the right.' },
    { rule: 'ax', opts: ['ax', '⊗', '⅋'], why: '~B and B alone: an axiom.' },
    { rule: 'ax', opts: ['⅋', '⊗', 'ax'], why: '~A and A alone: an axiom.' }] },
  exp: { path: EXP_PATH.map((p, i) => ['s' + (i + 1), p[1], p[2], p[3], p[4]]), reasons: EXP_REASONS },
  ill: { path: [['s1', 0, 0, '1L'], ['s2', 1, 0, '&L₂'], ['s3', 2, 1, '⊸L', [0]], ['s4', 3, 0, 'ax'], ['s5', 4, 0, 'ax']], reasons: [
    { rule: '1L', opts: ['1L', '&L₁', 'ax'], why: 'A 1 among the hypotheses carries nothing: drop it.' },
    { rule: '&L₂', opts: ['&L₁', '&L₂', '⊸L'], why: 'C needs B, so take B from A & B.' },
    { rule: '⊸L', opts: ['ax', '⊸L', '&L₁'], why: 'B proves B on the left; C is then used on the right.' },
    { rule: 'ax', opts: ['ax', '1L', '⊸L'], why: 'B ⊢ B: an axiom.' },
    { rule: 'ax', opts: ['⊸L', 'ax', '&L₂'], why: 'C ⊢ C: an axiom.' }] },
};
// each lesson: worked, faded (its last steps left to the student), practice (from the rubric) (D56)
const LESSONS = [
  { t: 'Sequents and the axiom', key: 'first', intro: 'Hypotheses move right of ⊢ as duals: A, A ⊸ B ⊢ B is the one-sided ⊢ ~A, A ⊗ ~B, B. An axiom closes an atom beside its dual.', fade: 1, practice: 'illfirst', pnote: 'The same sequent, two-sided: find the split of ⊸L.', tour: true },
  { t: 'Invertible first', key: 'mll', intro: '⅋ is invertible: applying it never loses a proof. ⊗ is not: its split can go wrong.', fade: 2, practice: 'first', pnote: 'One split needs care: which formula goes left?' },
  { t: 'Exponentials', key: 'exp', intro: 'Any ? formula may be copied (?c), used once (?d) or dropped (?w). ! applies only when every other formula is a ? formula.', fade: 5, practice: 'exp', pnote: 'The data has one proof with exponentials: prove it now with no reasons shown.' },
  { t: 'Two sides', key: 'ill', intro: 'Intuitionistic sequents keep one formula right of ⊢. Rules are named by side: L acts on a hypothesis, R on the conclusion.', fade: 3, practice: 'illfirst', pnote: 'Here ⊸L needs a split.' },
  { t: 'When there is no proof', erroneous: true },
];
const STAGES = ['Worked example', 'Your turn', 'Practice'];
const PRACTICE = [
  ['first', 1, 'One split, two axioms'], ['illfirst', 1, 'The same, two-sided'], ['mll', 1, '⅋ first, then the split'], ['balance', 1, 'Provable?', true],
  ['exp', 2, 'Contraction, dereliction, promotion'], ['ill', 2, 'An additive choice, then ⊸L'], ['classical', 2, 'Provable?', true], ['ordinary', 2, 'Peirce\'s law, ordinary logic'],
  ['exhaustive', 3, 'Provable?', true], ['unknown', 3, 'The search gives up', true], ['wide', 1, 'Wide and finished: zoom and pan', 'View'],
];
const DIFF = ['', 'Easy', 'Medium', 'Hard'];
function renderFrame() {
  if (!app.learn) return;
  const r = app.rubric || 'tutorial';
  $$('.ftab').forEach(t => t.setAttribute('aria-selected', String(t.dataset.rub === r)));
  const b = $('#frame-body');
  if (r === 'tutorial') b.innerHTML = tutorialHTML();
  if (r === 'practice') b.innerHTML = '<h3>Practice</h3><p class="sub">One click opens a sequent in the row of proofs.</p>' +
    [1, 2, 3].map(d => '<h4 class="kw-label" style="color:var(--learn)">' + DIFF[d] + ' <span aria-hidden="true">' + '●'.repeat(d) + '○'.repeat(3 - d) + '</span></h4><ul class="prac">' + PRACTICE.filter(x => x[1] === d).map(([k, , note, kind]) =>
      '<li><button type="button" data-ex="' + k + '">' + mathHTML(P[k].uni) + '<span class="diff">' + (kind === true ? 'Decide' : kind ? kind : P[k].mode === 'intuitionistic' ? 'Two-sided' : P[k].ordinary ? 'Ordinary' : 'Prove') + '</span><span class="d">' + fmtText(note) + '</span></button></li>').join('') + '</ul>').join('');
  if (r === 'reference') b.innerHTML = '<h3>Reference</h3><table class="ref-c"><thead><tr><th></th><th>Name</th><th>Reading</th><th>Dual</th></tr></thead><tbody>' +
    CONNECTIVES.map(([s, n, rd, d]) => '<tr><td>' + mathHTML(s) + '</td><td>' + esc(n.charAt(0).toUpperCase() + n.slice(1)) + '</td><td>' + fmtText(rd.charAt(0).toUpperCase() + rd.slice(1)) + '</td><td>' + (d ? mathHTML(d) : '') + '</td></tr>').join('') + '</tbody></table>' +
    '<h4>Rules</h4><p class="sub">One-sided on the left, two-sided on the right.</p><div class="rules-ref">' + RULES.map(R => '<div class="rcard"><div class="nm"><span class="m" style="font-size:18px">' + esc(R.r) + '</span><span class="sub">two-sided: ' + R.two.map(t => '<span class="m">' + esc(t[0]) + '</span>').join(', ') + '</span></div><div>' + schemaHTML(R.one[0], R.one[1], R.r) + '</div><div>' + R.two.map(([n, p, c]) => schemaHTML(p, c, n)).join('') + '</div></div>').join('') + '</div>';
  $$('.ftab').forEach(t => t.onclick = () => setRubric(t.dataset.rub));
  $$('[data-ex]', b).forEach(x => x.addEventListener('click', () => load(x.dataset.ex)));
  $$('[data-lesson]', b).forEach(x => x.addEventListener('click', () => { app.lesson = { i: +x.dataset.lesson, s: 0 }; app.worked = null; app.reason = null; layoutReasons(); renderFrame(); }));
  $$('[data-ls]', b).forEach(x => x.addEventListener('click', () => lessonAct(x.dataset.ls, x.dataset.v)));
  wireWorked(b);
}
function tutorialHTML() {
  const L = app.lesson || { i: 0, s: 0 }; app.lesson = L;
  const les = LESSONS[L.i];
  let h = '<h3>Tutorial</h3><ol class="lessons">' + LESSONS.map((x, i) => '<li><button type="button" data-lesson="' + i + '" aria-current="' + (i === L.i) + '">' + (i + 1) + '. ' + esc(x.t || 'When there is no proof') + '</button></li>').join('') + '</ol>';
  if (les.erroneous) return h + erroneousHTML(L);
  const stages = '<ol class="lesson-stages" aria-label="Stages">' + STAGES.map((s, i) => '<li' + (i === L.s ? ' aria-current="step"' : '') + (i < L.s ? ' class="done"' : '') + '>' + (i + 1) + '. ' + s + '</li>').join('') + '</ol>';
  h += '<div class="lesson"><p class="kw-label" style="color:var(--learn)">Lesson ' + (L.i + 1) + ': ' + esc(les.t) + '</p>' + stages;
  const done = app.proof === (L.s === 2 ? les.practice : les.key) && state() && state().done;
  if (L.s === 0) {
    h += '<p>' + fmtText(les.intro) + '</p>';
    if (app.worked && app.worked.key === les.key) h += workedHTML();
    else h += '<div class="row"><button type="button" class="btn btn-primary" data-ls="work">Play the proof</button>' + (les.tour ? '<button type="button" class="btn" data-ls="tour">Tour of the page</button>' : '') + '</div>';
  } else if (L.s === 1) {
    const left = PATHS[les.key].path.length - les.fade;
    h += '<p>The same proof, stopped ' + left + ' step' + (left > 1 ? 's' : '') + ' before the end. Finish it on the stage; H gives a hint.</p>';
    h += '<ul class="checks">' + PATHS[les.key].path.slice(les.fade).map((p, i) => '<li class="' + (app.proof === les.key && stepsDone(les.key) > les.fade + i ? 'done' : '') + '">' + fmtText('Step ' + (les.fade + i + 1) + ': ' + p[3] + ' on goal ' + p[1]) + '</li>').join('') + '<li class="' + (done ? 'done' : '') + '">Proof checked</li></ul>';
    h += '<div class="row"><button type="button" class="btn btn-primary" data-ls="fade">Open it, ' + left + ' step' + (left > 1 ? 's' : '') + ' left</button></div>';
  } else {
    h += '<p>From Practice: ' + mathHTML(P[les.practice].uni) + '. ' + fmtText(les.pnote) + '</p><ul class="checks"><li class="' + (done ? 'done' : '') + '">Proof checked</li></ul>';
    h += '<div class="row"><button type="button" class="btn btn-primary" data-ls="practice">Open the problem</button></div>';
  }
  h += '<div class="row" style="margin-top:8px"><span class="grow"></span>' + (L.s > 0 ? '<button type="button" class="btn" data-ls="back">Back</button>' : '') + (L.s < 2 ? '<button type="button" class="btn" data-ls="next">Next: ' + STAGES[L.s + 1].toLowerCase() + '</button>' : '') + '</div></div>';
  return h;
}
function stepsDone(key) { const p = PATHS[key].path.map(x => x[0]); return p.indexOf(app.sk) + 1; }
// lesson 5: the erroneous example, then the refutation (D56)
function erroneousHTML(L) {
  const stages = '<ol class="lesson-stages" aria-label="Stages">' + ['Find the wrong step', 'The refutation'].map((s, i) => '<li' + (i === L.s ? ' aria-current="step"' : '') + (i < L.s ? ' class="done"' : '') + '>' + (i + 1) + '. ' + s + '</li>').join('') + '</ol>';
  let h = '<div class="lesson"><p class="kw-label" style="color:var(--learn)">Lesson 5: When there is no proof</p>' + stages;
  if (L.s === 0) {
    h += '<p>Every step of this derivation is legal, yet goals 2 and 3 cannot close. Which step went wrong?</p>';
    const pick = L.pick;
    const opts = [['par', '⅋ on goal 0'], ['tensor', '⊗ on goal 1, sending ~A left']];
    h += '<div class="choice">' + opts.map(([v, t]) => '<button type="button" data-ls="pick" data-v="' + v + '" class="' + (pick ? (v === 'tensor' ? 'right' : v === pick ? 'wrong' : '') : '') + '">' + fmtText(t) + '</button>').join('') + '</div>';
    if (pick) h += '<p>' + fmtText(pick === 'tensor' ? 'Yes. ~A went left, away from A: each premise keeps an atom without its dual.' : 'Not this one: ⅋ is invertible and never goes wrong. Look at the split.') + '</p>';
    if (app.proof !== 'mll' || app.sk !== 'd1') h += '<div class="row"><button type="button" class="btn btn-primary" data-ls="dead">Open the derivation</button></div>';
  } else {
    h += '<p>The search can say why goal 2 has no proof: count the atoms. Close goal 2, then read its verdict at the goal.</p><div class="row"><button type="button" class="btn btn-primary" data-ls="refute">Close goal 2</button><button type="button" class="btn" data-ls="whole">The same for A ⊗ B ⊢ A</button></div>';
  }
  h += '<div class="row" style="margin-top:8px"><span class="grow"></span>' + (L.s > 0 ? '<button type="button" class="btn" data-ls="back">Back</button>' : '<button type="button" class="btn" data-ls="next">Next: the refutation</button>') + '</div></div>';
  return h;
}
function lessonAct(a, v) {
  const L = app.lesson, les = LESSONS[L.i];
  if (a === 'work') startWorked(les.key);
  if (a === 'tour') startTour(0);
  if (a === 'fade') { load(les.key, { quiet: true }); scriptSteps(PATHS[les.key].path.slice(0, les.fade)); app.worked = null; app.reason = null; layoutReasons(); }
  if (a === 'practice') load(les.practice);
  if (a === 'next') { L.s++; app.worked = null; app.reason = null; layoutReasons(); if (les.erroneous && L.s === 1 && !(app.proof === 'mll' && app.sk === 'd1')) dead(); }
  if (a === 'back') { L.s--; app.worked = null; app.reason = null; layoutReasons(); }
  if (a === 'dead') dead();
  if (a === 'pick') L.pick = v;
  if (a === 'refute') { if (!(app.proof === 'mll' && app.sk === 'd1')) dead(); app.active = 2; markActive(); closeGoal(false); }
  if (a === 'whole') { load('balance'); closeGoal(false); }
  renderFrame();
}
function lessonWatch() { if (app.learn && (app.rubric || 'tutorial') === 'tutorial') renderFrame(); }
// the worked example: predict, reveal, the reason pinned at the inference (D21)
function startWorked(key) { load(key, { quiet: true }); app.worked = { key, i: 0, picked: null }; app.reason = null; layoutReasons(); renderFrame(); }
function workedHTML() {
  const w = app.worked; if (!w) return '';
  const P2 = PATHS[w.key], n = P2.path.length;
  if (w.i >= n) return '<p class="sub">All ' + n + ' steps played. Next: your turn, on the same proof.</p>';
  const R = P2.reasons[w.i];
  const opts = R.opts.map(o => '<button type="button" data-o="' + esc(o) + '" class="' + (w.picked ? (o === R.rule ? 'right' : o === w.picked ? 'wrong' : '') : '') + '"' + (w.picked ? ' disabled' : '') + '>' + esc(o) + '</button>').join('');
  return '<div class="predict"><p class="kw-label" style="color:var(--learn)">Predict the next rule on goal ' + P2.path[w.i][1] + ' (step ' + (w.i + 1) + ' of ' + n + ')</p><div class="opts">' + opts + '</div>' +
    (w.picked ? '<p>' + (w.picked === R.rule ? 'Yes. ' : 'It is ' + ruleHTML(R.rule) + '. ') + fmtText(R.why) + '</p><button type="button" class="btn btn-primary" data-w2="next">Next step</button>' : '<button type="button" class="btn btn-s" data-w2="show">Show it</button>') + '</div>';
}
function wireWorked(b) {
  $$('[data-o]', b).forEach(x => x.addEventListener('click', () => { app.worked.picked = x.dataset.o; revealWorked(); }));
  $$('[data-w2]', b).forEach(x => x.addEventListener('click', () => {
    if (x.dataset.w2 === 'show') { app.worked.picked = '—'; revealWorked(); }
    if (x.dataset.w2 === 'next') { app.worked.i++; app.worked.picked = null; app.reason = null; layoutReasons(); renderFrame(); }
  }));
}
function revealWorked() {
  const w = app.worked, p = PATHS[w.key].path[w.i];
  app.hist.push({ from: app.sk, to: p[0], goal: p[1], pos: p[2], rule: p[3], left: p[4], by: 'lesson' });
  go(p[0], true);
  const conc = Object.keys(geo.conc).filter(id => geo.conc[id].info && geo.conc[id].info.rule === p[3]).sort((a, b) => geo.conc[a].y - geo.conc[b].y)[0];
  app.reason = { conc, rule: p[3], why: PATHS[w.key].reasons[w.i].why };
  layoutReasons();
  renderFrame();
}
function scriptSteps(list) {
  list.forEach(([to, goal, pos, rule, left]) => { app.hist.push({ from: app.sk, to, goal, pos, rule, left, by: 'you' }); app.sk = to; });
  app.active = null;
  go(app.sk, false);
}

// ---------- tour (D8): a live spotlight on README's first example
const TOUR = [
  { t: () => $('#proofs .ptab[aria-current="true"]'), text: 'Each tab is a proof. The tour opened README\'s first example, A, A ⊸ B ⊢ B; click its text to edit it, its mark for the calculus.', act: () => load('first') },
  { t: () => hits.querySelector('.fb[data-pos="1"]'), text: 'Every formula is a button. This is A ⊗ ~B, the dual of the hypothesis A ⊸ B.' },
  { t: () => $('#pop'), text: 'One click lists the rules for that formula, beside the goal.', act: () => { const b = hits.querySelector('.fb[data-pos="1"]'); onFormula(0, 1, b); } },
  { t: () => $('#pop'), text: '⊗ asks for a split: ~A goes left. The popover says whether the split can close.', act: () => { if (!app.split) pickRule('⊗'); const b = hits.querySelector('.fb[data-pos="0"]'); if (!app.split.left.has(0)) toggleSplit(0, 0, b); } },
  { t: () => $('#dock'), text: 'Two goals. What acts on a goal sits at the goal: its number, the way to the next one, Close goal.', act: () => { if (app.split) applySplit(); } },
  { t: () => activeDot(), text: 'Both axioms applied. The tab\'s dot is the verdict: green means proved and checked. Hover it for the details.', act: () => { if (app.sk === 's1') step({ to: 's2', goal: 1, pos: 0, rule: 'ax' }); if (app.sk === 's2') step({ to: 's3', goal: 2, pos: 0, rule: 'ax' }); } },
  { t: () => $('#bottom'), text: 'What you can do with the proof is in the bottom band. Every key is in the tooltips and under Keys.' },
];
let tourI = -1;
function startTour(i) {
  closePop(); closeMenu(); closeCalc();
  tourI = i || 0;
  for (let j = 0; j <= tourI; j++) if (TOUR[j].act) TOUR[j].act();
  $('#tour').hidden = false;
  showTour();
}
function showTour() {
  const s = TOUR[tourI];
  requestAnimationFrame(() => requestAnimationFrame(() => {
    const el = s.t();
    const r = el && !el.hidden ? el.getBoundingClientRect() : { left: innerWidth / 2 - 100, top: innerHeight / 2 - 50, width: 200, height: 100, right: innerWidth / 2 + 100, bottom: innerHeight / 2 + 50 };
    Object.assign($('#tour-hole').style, { left: (r.left - 6) + 'px', top: (r.top - 6) + 'px', width: (r.width + 12) + 'px', height: (r.height + 12) + 'px' });
    const cap = $('#tour-cap');
    cap.innerHTML = '<p class="kw-label"><span>Tour</span><span>' + (tourI + 1) + ' of ' + TOUR.length + '</span></p><p>' + fmtText(s.text) + '</p><div class="row"><button type="button" class="btn" id="t-end" data-key="Esc">End</button>' + (tourI ? '<button type="button" class="btn" id="t-back" data-key="←">Back</button>' : '') + '<button type="button" class="btn btn-primary" id="t-next" data-key="→">' + (tourI === TOUR.length - 1 ? 'Finish' : 'Next') + '</button></div>';
    const cw = cap.offsetWidth, ch = cap.offsetHeight;
    let x = r.left, y = r.bottom + 14;
    if (y + ch > innerHeight - 8) y = r.top - ch - 14;
    if (y < 8) { y = Math.min(innerHeight - ch - 8, r.top + 8); x = r.right + 14; if (x + cw > innerWidth - 8) x = r.left - cw - 14; }
    cap.style.left = Math.max(8, Math.min(x, innerWidth - cw - 8)) + 'px'; cap.style.top = Math.max(8, y) + 'px';
    $('#t-next').addEventListener('click', tourNext); $('#t-end').addEventListener('click', () => endTour());
    const bk = $('#t-back'); if (bk) bk.addEventListener('click', tourBack);
    $('#t-next').focus();
  }));
}
function tourNext() { if (tourI >= TOUR.length - 1) return endTour(); tourI++; if (TOUR[tourI].act) TOUR[tourI].act(); showTour(); }
function tourBack() { if (tourI <= 0) return; const t = tourI - 1; endTour(true); startTour(t); }
function endTour(silent) { $('#tour').hidden = true; tourI = -1; if (!silent) announce('Tour ended.'); }

// ---------- keys (D17): every key here and in the tooltips (D55)
const KEYS = [
  ['Tab', 'Next formula or control'], ['Enter', 'Rules of the focused formula'], ['1–9', 'Pick a rule'], ['Esc', 'Close, cancel a split or an edit'],
  ['U', 'Undo on this branch'], ['⇧U', 'Redo'], ['S', 'Steps'], ['C', 'Close goal'], ['⇧C', 'Close all'], ['H', 'Next hint rung'],
  ['[ ]', 'Previous or next goal'], ['G', 'Goal into view'], ['+ −', 'Zoom'], ['F', 'Fit the tree'], ['⇧ arrows', 'Pan'], ['V', 'Tree or net'],
  ['N', 'New proof'], ['/', 'Edit the sequent'], ['E', 'Export'], ['I', 'Import'], [',', 'Settings'], ['L', 'Learn'], ['T', 'Tour'], ['A', 'About'], ['?', 'This list'],
];
function toggleKeys(on) {
  const k = $('#keys');
  const show = on === undefined ? k.hidden : on;
  k.hidden = !show;
  if (show) {
    k.innerHTML = '<h2 id="keys-t" class="kw-label">Keys <button type="button" class="btn icon-b" id="keys-x" aria-label="Close" data-tip="Close" data-key="Esc"><svg class="ic" aria-hidden="true"><use href="#i-x"/></svg></button></h2><dl>' + KEYS.map(([a, b]) => '<dt>' + a.split(' ').map(x => '<kbd>' + esc(x) + '</kbd>').join(' ') + '</dt><dd>' + esc(b) + '</dd>').join('') + '</dl><p class="foot">Settings can show the keys beside the actions. The page is installable and works offline after one visit.</p>';
    $('#keys-x').addEventListener('click', () => toggleKeys(false));
    $('#keys-x').focus();
  }
}

// ---------- toasts, live region
let toastT = null;
function toast(msg) { const t = $('#toast'); t.textContent = msg; t.classList.add('on'); clearTimeout(toastT); toastT = setTimeout(() => t.classList.remove('on'), 2600); }
function announce(msg) { const l = $('#live'); l.textContent = ''; setTimeout(() => { l.textContent = msg; }, 30); }
function relayout() { if (geo) (state() && !state().done ? focusGoal(false) : fit(false)); if (tourI >= 0) showTour(); placeFieldMsg(); }

// ---------- wiring
$('#seq-form').addEventListener('submit', e => { e.preventDefault(); submitField(); });
$('#field').addEventListener('input', () => { if ($('#field').getAttribute('aria-invalid') === 'true') setFieldMsg(''); });
$('#field').addEventListener('keydown', e => { if (e.key === 'Escape' && app.tabs.length) { e.preventDefault(); e.stopPropagation(); cancelEdit(); } });
$$('[data-part]').forEach(b => b.addEventListener('click', () => setPart(b.dataset.part)));
$$('[data-mode]').forEach(b => b.addEventListener('click', () => { if (b.dataset.mode !== app.mode) requestMode({ mode: b.dataset.mode }); }));
$('#mix').addEventListener('change', e => requestMode({ mix: e.target.checked }));
$('#affine').addEventListener('change', e => requestMode({ affine: e.target.checked }));
$('#mode-go').addEventListener('click', () => applyMode(pendingMode));
$('#mode-keep').addEventListener('click', () => { $('#mode-warn').hidden = true; pendingMode = null; syncModeUI(); });
$('#undo').addEventListener('click', undo); $('#redo').addEventListener('click', redo);
$('#closeall').addEventListener('click', () => (app.retry || (app.verdict && app.verdict.word === 'Unknown')) ? retry() : closeGoal(true));
$('#hint').addEventListener('click', hint);
$('#tab-tree').addEventListener('click', () => setView('tree')); $('#tab-net').addEventListener('click', () => setView('net'));
$('#zoom-in').addEventListener('click', () => zoomAt(1.2)); $('#zoom-out').addEventListener('click', () => zoomAt(1 / 1.2));
$('#fit').addEventListener('click', () => fit(true));
$('#learn').addEventListener('click', () => toggleLearn()); $('#learn-x').addEventListener('click', () => toggleLearn(false));
$('#tour-b').addEventListener('click', () => startTour(0));
$('#keys-b').addEventListener('click', () => toggleKeys());
$('#export-b').addEventListener('click', () => openMenu('export'));
$('#import-b').addEventListener('click', () => openMenu('import'));
$('#settings-b').addEventListener('click', () => openMenu('settings'));
$('#steps-b').addEventListener('click', () => openMenu('steps'));
$('#about-b').addEventListener('click', () => openMenu('about'));
$('#theme-b').addEventListener('click', () => setTheme(currentDark() ? 'light' : 'dark'));
$('#status-pop').addEventListener('mouseleave', () => { if (!statusPinned) closeStatus(); });
$('#sheet-handle').addEventListener('click', () => { const b = $('#bottom'); const o = !b.classList.contains('open'); b.classList.toggle('open', o); $('#sheet-handle').setAttribute('aria-expanded', String(o)); setTimeout(relayout, 30); });
document.addEventListener('pointerdown', e => {
  if (!menuEl.hidden && !e.target.closest('#menu, #export-b, #import-b, #settings-b, #steps-b, #about-b')) closeMenu();
  if (statusPinned && !e.target.closest('#status-pop, .pt-dot')) closeStatus();
  if (!$('#calc-pop').hidden && !e.target.closest('#calc-pop, .pt-mark')) closeCalc();
});
addEventListener('resize', relayout);
new ResizeObserver(() => relayout()).observe(stage);

document.addEventListener('keydown', e => {
  const tag = (e.target.tagName || '').toLowerCase();
  const typing = tag === 'input' || tag === 'textarea';
  if (e.key === 'Escape') {
    if (tourI >= 0) return endTour();
    if (!$('#keys').hidden) return toggleKeys(false);
    if (!$('#calc-pop').hidden) { closeCalc(); const b = $('#proofs .ptab[aria-current="true"] .pt-mark'); if (b) b.focus(); return; }
    if (!menuEl.hidden) { const a = { export: '#export-b', import: '#import-b', settings: '#settings-b', steps: '#steps-b', about: '#about-b' }[app.menu]; closeMenu(); if (a) $(a).focus(); return; }
    if (!$('#status-pop').hidden) return closeStatus();
    if (!pop.hidden) { const s = app.sel; closePop(); if (s && s.btn && s.btn.isConnected) s.btn.focus(); return; }
    if (app.retry) return stopSearch();
    if (app.dockLine) return setDockLine(null);
    return;
  }
  if (tourI >= 0) { if (e.key === 'ArrowRight' || e.key === 'Enter') { e.preventDefault(); tourNext(); } if (e.key === 'ArrowLeft') tourBack(); return; }
  if (typing) return;
  if (!pop.hidden && /^[1-9]$/.test(e.key)) { const b = $$('.rules button', pop)[+e.key - 1]; if (b) { e.preventDefault(); b.click(); } return; }
  if (app.split && e.key === 'Enter' && !e.target.closest('.fb')) { e.preventDefault(); applySplit(); return; }
  if (e.metaKey || e.ctrlKey || e.altKey) return;
  const k = e.key;
  if (e.shiftKey && k.startsWith('Arrow')) { const d = 60; if (k === 'ArrowLeft') app.cam.x += d; if (k === 'ArrowRight') app.cam.x -= d; if (k === 'ArrowUp') app.cam.y += d; if (k === 'ArrowDown') app.cam.y -= d; applyCam(true); e.preventDefault(); return; }
  const map = {
    u: undo, U: redo, s: () => openMenu('steps'), c: () => closeGoal(false), C: () => $('#closeall').click(), h: hint, '[': () => cycleGoal(-1), ']': () => cycleGoal(1),
    g: () => focusGoal(true), '+': () => zoomAt(1.2), '=': () => zoomAt(1.2), '-': () => zoomAt(1 / 1.2), f: () => fit(true), v: () => setView(app.view === 'tree' ? 'net' : 'tree'),
    e: () => openMenu('export'), i: () => openMenu('import'), ',': () => openMenu('settings'), l: () => toggleLearn(), t: () => startTour(0), '?': () => toggleKeys(),
    '/': () => (app.proof ? editTab() : newProof()), n: newProof, a: () => openMenu('about'), '`': () => toggleScenes(),
  };
  if (map[k]) { e.preventDefault(); map[k](); }
});

// ---------- scenes for the reviewer (?scene=NAME, or the corner control / `)
function openAt(g, p) { const b = hits.querySelector('.fb[data-goal="' + g + '"][data-pos="' + p + '"]'); if (b) onFormula(g, p, b); }
function toggleAt(g, p) { const b = hits.querySelector('.fb[data-goal="' + g + '"][data-pos="' + p + '"]'); if (b) toggleSplit(g, p, b); }
function later(f) { setTimeout(f, 120); }
function tipOn(sel) { later(() => { const el = typeof sel === 'string' ? $(sel) : sel(); if (el) showTip(el); }); }
const MLL_DONE = PATHS.mll.path;
const expTo = n => PATHS.exp.path.slice(0, n);
const ILL_PATH = PATHS.ill.path;
function dead() { load('mll', { quiet: true }); scriptSteps([['s1', 0, 0, '⅋'], ['d1', 1, 2, '⊗', [0]]]); }
function lesson(i, s, extra) { toggleLearn(true); setRubric('tutorial'); app.lesson = Object.assign({ i, s }, extra || {}); renderFrame(); }
const SC = {
  open: () => newProof(),
  'mll-0': () => load('mll'),
  'mll-pop': () => { load('mll'); openAt(0, 0); },
  'mll-1': () => { load('mll'); scriptSteps(MLL_DONE.slice(0, 1)); },
  'mll-split-wrong': () => { SC['mll-1'](); openAt(1, 2); pickRule('⊗'); toggleAt(1, 0); },
  'mll-split': () => { SC['mll-1'](); openAt(1, 2); pickRule('⊗'); toggleAt(1, 1); },
  'mll-2': () => { load('mll'); scriptSteps(MLL_DONE.slice(0, 2)); },
  'mll-done': () => { load('mll'); scriptSteps(MLL_DONE); },
  'mll-net': () => { SC['mll-done'](); setView('net'); },
  'dead-split': dead,
  'dead-refused': () => { dead(); refuse(2, 0, AX0, 'ax'); },
  'refusal-tip': () => { dead(); refuse(2, 0, AX0, 'ax'); tipOn(() => $('.dline .kw')); },
  'dead-close': () => { dead(); app.active = 2; markActive(); closeGoal(false); tipOn(() => $('.dline .kw')); },
  'dead-undo': () => { dead(); app.active = 2; closeGoal(false); undo(); },
  'close-before': () => SC['mll-1'](),
  'close-done': () => { SC['mll-1'](); closeGoal(false); later(() => openStatus(true)); },
  'exp-0': () => load('exp'),
  'exp-pop': () => { load('exp'); openAt(0, 0); },
  'exp-3': () => { load('exp'); scriptSteps(expTo(3)); },
  'exp-done': () => { load('exp'); scriptSteps(expTo(7)); },
  'exp-net': () => { SC['exp-done'](); setView('net'); },
  'ill-0': () => load('ill'),
  'ill-pop': () => { load('ill'); scriptSteps(ILL_PATH.slice(0, 1)); openAt(1, 0); },
  'ill-done': () => { load('ill'); scriptSteps(ILL_PATH); },
  'unprov-balance': () => { load('balance'); closeGoal(false); later(() => openStatus(true)); },
  'unprov-classical': () => { load('classical'); closeGoal(false); later(() => openStatus(true)); },
  'unprov-exhaustive': () => { load('exhaustive'); closeGoal(false); later(() => openStatus(true)); },
  'status-pop': () => SC['unprov-classical'](),
  unknown: () => { load('unknown'); closeGoal(false); later(() => openStatus(true)); },
  'unknown-retry': () => { load('unknown'); closeGoal(false); retry(); later(() => openStatus(true)); },
  'parse-star': () => { SC['mll-1'](); editTab(); $('#field').value = 'A ** B |- A'; submitField(); tipOn(() => $('#field-msg .kw')); },
  'parse-missing': () => { newProof(); $('#field').value = 'A * |- B'; submitField(); },
  'parse-ill': () => { app.mode = 'intuitionistic'; syncModeUI(); newProof(); $('#field').value = 'A |- B, C'; submitField(); },
  'edit-tab': () => { SC['mll-2'](); editTab(); },
  calc: () => { SC['mll-2'](); openCalc(); },
  'mode-warn': () => { load('first'); scriptSteps([['s1', 0, 1, '⊗', [0]]]); openCalc(); requestMode({ mode: 'intuitionistic' }); },
  'mode-ill': () => load('illfirst'),
  proofs: () => { load('ill'); scriptSteps(ILL_PATH.slice(0, 1)); load('exp'); scriptSteps(expTo(3)); load('mll'); scriptSteps(MLL_DONE.slice(0, 2)); },
  'proofs-switch': () => { SC.proofs(); switchTab(1); },
  'new-tab': () => { SC.proofs(); newProof(); },
  learn: () => { load('first'); toggleLearn(true); },
  'tut-1': () => { load('first'); lesson(0, 0); },
  'tut-2': () => { lesson(1, 0); startWorked('mll'); },
  'tut-2-worked': () => { lesson(1, 0); startWorked('mll'); app.worked.picked = '⊗'; revealWorked(); },
  'tut-2-faded': () => { lesson(1, 1); lessonAct('fade'); },
  'tut-2-progress': () => { lesson(1, 1); lessonAct('fade'); step({ to: 's3', goal: 2, pos: 0, rule: 'ax' }, true); },
  'tut-2-practice': () => { lesson(1, 2); lessonAct('practice'); },
  'tut-3': () => { load('exp'); lesson(2, 0); },
  'tut-5': () => { lesson(4, 0); dead(); renderFrame(); },
  'tut-5-found': () => { lesson(4, 0, { pick: 'tensor' }); dead(); renderFrame(); },
  'tut-5-refute': () => { lesson(4, 1); lessonAct('refute'); tipOn(() => $('.dline .kw')); },
  'worked-predict': () => { lesson(2, 0); startWorked('exp'); },
  worked: () => { lesson(2, 0); startWorked('exp'); app.worked.picked = '?d'; revealWorked(); },
  practice: () => { load('mll'); toggleLearn(true); setRubric('practice'); },
  reference: () => { load('mll'); toggleLearn(true); setRubric('reference'); },
  'learn-rule': () => { load('mll'); toggleLearn(true); scriptSteps(MLL_DONE.slice(0, 1)); setRubric('reference'); openAt(1, 2); tipOn(() => $('.rules button')); },
  'learn-connective': () => { load('mll'); toggleLearn(true); tipOn(() => hits.querySelector('.fb[data-pos="0"]')); },
  'hint-1': () => { SC['mll-1'](); hint(); },
  'hint-2': () => { SC['mll-1'](); hint(); hint(); },
  'hint-3': () => { SC['mll-1'](); hint(); hint(); hint(); },
  'hint-4': () => { SC['mll-1'](); hint(); hint(); hint(); hint(); later(() => { const b = $('[data-h="why"]'); if (b) b.click(); }); },
  'tour-1': () => startTour(0), 'tour-3': () => startTour(2), 'tour-4': () => startTour(3), 'tour-6': () => startTour(5),
  exports: () => { SC['mll-done'](); openMenu('export'); },
  'exports-link': () => { SC['mll-2'](); openMenu('export', 'link'); },
  'exports-cert': () => { load('balance'); closeGoal(false); closeStatus(); openMenu('export', 'cert'); },
  steps: () => { SC['mll-done'](); undo(); openMenu('steps'); },
  import: () => { SC['mll-0'](); openMenu('import'); },
  'import-paste': () => { load('first'); openMenu('import'); $('#imp-text').value = SAMPLES.session(); importText(SAMPLES.session(), 'paste'); },
  'import-level': () => { SC['mll-0'](); openMenu('import'); $('#imp-text').value = SAMPLES.level(); importText(SAMPLES.level(), 'paste'); },
  'import-drop': () => { SC['mll-0'](); $('#drop').hidden = false; },
  'import-dropped': () => { const r = importText(SAMPLES.sequent(), 'drop'); showDropResult(r); },
  settings: () => { SC['mll-2'](); openMenu('settings'); },
  'keys-on': () => { setShowKeys(true); SC['mll-2'](); },
  'keys-off': () => { setShowKeys(false); SC['mll-2'](); },
  about: () => { SC['mll-done'](); openMenu('about'); },
  'link-broken': () => { load('mll'); const n = document.createElement('div'); n.className = 'cluster notice'; n.setAttribute('role', 'status'); n.innerHTML = '<button type="button" class="kw warn" data-tip="The link\'s session could not be read, perhaps made by another version of linlog. The sequent and the calculus are loaded; the steps are not.">Proof not restored</button><span class="sub">Sequent and calculus loaded</span><button type="button" class="btn icon-b" id="notice-x" aria-label="Dismiss" data-tip="Dismiss"><svg class="ic" aria-hidden="true"><use href="#i-x"/></svg></button>'; stage.appendChild(n); $('#notice-x').addEventListener('click', () => n.remove()); tipOn(() => $('.notice .kw')); },
  keys: () => { load('mll'); later(() => { const b = hits.querySelector('.fb'); if (b) b.focus(); toggleKeys(true); }); },
  focus: () => { SC['mll-2'](); later(() => { const b = hits.querySelector('.fb'); if (b) b.focus(); }); },
  'focus-undo': () => { SC['mll-2'](); later(() => $('#undo').focus()); },
  wide: () => { load('wide'); later(() => zoomAt(1.8, 160, 260)); },
  'wide-net': () => { load('wide'); setView('net'); },
  ordinary: () => load('ordinary'),
  sheet: () => { SC['mll-2'](); $('#sheet-handle').click(); },
};
const SCENE_GROUPS = [
  ['Opening', ['open']],
  ['1 Proving A ⊗ B ⊢ B ⊗ A', ['mll-0', 'mll-pop', 'mll-1', 'mll-split-wrong', 'mll-split', 'mll-2', 'mll-done', 'mll-net']],
  ['2 Dead end', ['dead-split', 'dead-refused', 'refusal-tip', 'dead-close', 'dead-undo']],
  ['3 Search closes', ['close-before', 'close-done']],
  ['4 Second proofs', ['exp-0', 'exp-pop', 'exp-3', 'exp-done', 'ill-0', 'ill-pop', 'ill-done']],
  ['5 Unprovable', ['unprov-balance', 'unprov-classical', 'unprov-exhaustive']],
  ['6 Gave up', ['unknown', 'unknown-retry']],
  ['7 Parse', ['parse-star', 'parse-missing', 'parse-ill']],
  ['8 Learn', ['tut-1', 'tut-2', 'tut-2-worked', 'tut-2-faded', 'tut-2-progress', 'tut-2-practice', 'tut-3', 'tut-5', 'tut-5-found', 'tut-5-refute', 'worked-predict', 'worked', 'practice', 'reference', 'learn-rule', 'learn-connective']],
  ['9 Hints', ['hint-1', 'hint-2', 'hint-3', 'hint-4']],
  ['10 Tour', ['tour-1', 'tour-3', 'tour-4', 'tour-6']],
  ['11 Export, import', ['exports', 'exports-link', 'exports-cert', 'steps', 'import', 'import-paste', 'import-level', 'import-drop', 'import-dropped', 'link-broken', 'settings', 'about']],
  ['12 Calculus', ['calc', 'mode-warn', 'mode-ill', 'exp-net']],
  ['13 Keys, phone', ['keys', 'keys-on', 'keys-off', 'focus', 'focus-undo', 'sheet']],
  ['14 Camera', ['wide', 'wide-net', 'mll-2']],
  ['16 Ordinary', ['ordinary']],
  ['Several proofs', ['proofs', 'proofs-switch', 'new-tab', 'edit-tab']],
];
function toggleScenes() {
  const s = $('#scenes');
  s.hidden = !s.hidden;
  if (!s.hidden && !s.innerHTML) {
    const th = new URLSearchParams(location.search).get('theme');
    s.innerHTML = SCENE_GROUPS.map(([h, list]) => '<h2>' + esc(h) + '</h2>' + list.map(n => '<a href="?scene=' + n + (th ? '&theme=' + th : '') + '">' + n + '</a>').join(' ')).join('') + '<h2>15 Theme</h2><a href="?theme=light">light</a> <a href="?theme=dark">dark</a> <a href="?">system</a>';
  }
}
$('#scenes-b').addEventListener('click', toggleScenes);

// ---------- start: with no proof open, the field is focused in a new tab (D55)
function start() { try { start0(); } catch (e) { if (DEBUG) document.body.dataset.err = String(e && e.stack || e).replace(/\s+/g, " ").slice(0, 300); else throw e; } }
function start0() {
  setTheme(document.documentElement.getAttribute('data-theme') || 'system');
  clearStage(); renderViewTabs();
  const q = new URLSearchParams(location.search), sc = q.get('scene');
  if (DEBUG && q.has('throwtest')) throw new Error('throwtest');
  if (sc && SC[sc] && sc !== 'open') SC[sc]();
  else newProof();
}
if (document.fonts && document.fonts.load) {
  Promise.all([document.fonts.load('1000px "Euler Math"', '𝐴⊢⊗'), document.fonts.load('14px Figtree'), document.fonts.load('14px "Space Mono"')]).then(start, start);
} else start();
})();
