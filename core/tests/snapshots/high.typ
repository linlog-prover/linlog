#set page(width: auto, height: auto, margin: 5pt)

#context {
  let open = "dots"
  let gap = (1.5em).to-absolute()
  let name-gap = (0.2em).to-absolute()
  let band = (0.8em).to-absolute()
  let stroke = 0.05em
  let dots = $dots.v$
  let nodes = (
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (1, $⊥$, $⊢ ⊥, bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (2, $⊗$, $⊢ bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))))$),
    (0, $bold(1)$, $⊢ bold(1)$),
    (2, $⊗$, $⊢ bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1)))))))))$),
    (0, $bold(1)$, $⊢ bold(1)$),
    (2, $⊗$, $⊢ bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))))$),
    (0, $bold(1)$, $⊢ bold(1)$),
    (2, $⊗$, $⊢ bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1)))))))$),
    (0, $bold(1)$, $⊢ bold(1)$),
    (2, $⊗$, $⊢ bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))))$),
    (0, $bold(1)$, $⊢ bold(1)$),
    (2, $⊗$, $⊢ bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1)))))$),
    (0, $bold(1)$, $⊢ bold(1)$),
    (2, $⊗$, $⊢ bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1))))$),
    (0, $bold(1)$, $⊢ bold(1)$),
    (2, $⊗$, $⊢ bold(1) ⊗ (bold(1) ⊗ (bold(1) ⊗ bold(1)))$),
    (0, $bold(1)$, $⊢ bold(1)$),
    (2, $⊗$, $⊢ bold(1) ⊗ (bold(1) ⊗ bold(1))$),
    (0, $bold(1)$, $⊢ bold(1)$),
    (2, $⊗$, $⊢ bold(1) ⊗ bold(1)$),
    (0, $bold(1)$, $⊢ bold(1)$),
    (0, $bold(1)$, $⊢ bold(1)$),
  )
  let n = nodes.len()
  let tall = (3em).to-absolute()
  let metrics(c) = {
    let size = measure(box(c))
    let low = measure(box[#box(height: tall, width: 0pt)#c]).height - tall
    (size.width, size.height - low, low)
  }
  let (dots-width, dots-up, dots-down) = metrics(dots)
  let (width, up, down, wide, left, offset) = ((0pt,) * n,) * 6
  let bar = (none,) * n
  let names = (none,) * n
  let stack = ()
  for i in range(n - 1, -1, step: -1) {
    let (k, name, c) = nodes.at(i)
    let (w, a, d) = metrics(c)
    width.at(i) = w
    up.at(i) = a
    down.at(i) = d
    if k == none {
      let reach = if open == "dots" { dots-width } else { 0pt }
      wide.at(i) = calc.max(w, reach)
      left.at(i) = calc.max(reach - w, 0pt) / 2
      if open == "dashed" { bar.at(i) = (left.at(i), left.at(i) + w) }
      stack.push(i)
      continue
    }
    let kids = ()
    for _ in range(k) { kids.push(stack.pop()) }
    let x = 0pt
    let span = none
    for p in kids {
      offset.at(p) = x
      let s = x + left.at(p)
      span = (if span == none { s } else { span.at(0 ) }, s + width.at(p))
      x += wide.at(p) + gap
    }
    let (l, b) = if span == none { (0pt, (0pt, w)) } else {
      let l = (span.at(0) + span.at(1) - w) / 2
      (l, (calc.min(span.at(0), l), calc.max(span.at(1), l + w)))
    }
    let shift = calc.max(-l, 0pt)
    for p in kids { offset.at(p) += shift }
    let named = 0pt
    if name != none {
      let size = measure(box(name))
      names.at(i) = (size.width, size.height)
      named = name-gap + size.width
    }
    let row = if k == 0 { 0pt } else { x - gap }
    wide.at(i) = calc.max(row, b.at(1) + named) + shift
    left.at(i) = l + shift
    bar.at(i) = (b.at(0) + shift, b.at(1) + shift)
    stack.push(i)
  }
  let (x, depth) = ((0pt,) * n, (0,) * n)
  let parents = ()
  for i in range(n) {
    if parents.len() > 0 {
      let (p, r) = parents.last()
      x.at(i) = x.at(p) + offset.at(i)
      depth.at(i) = depth.at(p) + 1
      if r == 1 { let _ = parents.pop() } else { parents.at(-1) = (p, r - 1) }
    }
    let k = nodes.at(i).at(0)
    if k != none and k > 0 { parents.push((i, k)) }
  }
  let rows = calc.max(..depth) + 2
  let (row-up, row-down, between) = ((0pt,) * rows,) * 3
  for i in range(n) {
    let j = depth.at(i)
    row-up.at(j) = calc.max(row-up.at(j), up.at(i))
    row-down.at(j) = calc.max(row-down.at(j), down.at(i))
    if bar.at(i) != none {
      let tall = if names.at(i) == none { 0pt } else { names.at(i).at(1) }
      between.at(j) = calc.max(between.at(j), band, tall)
    }
    if nodes.at(i).at(0) == none and open == "dots" {
      between.at(j) = calc.max(between.at(j), band / 2)
      row-up.at(j + 1) = calc.max(row-up.at(j + 1), dots-up)
      row-down.at(j + 1) = calc.max(row-down.at(j + 1), dots-down)
    }
  }
  let base = (0pt,) * rows
  base.at(rows - 1) = row-up.at(rows - 1)
  for j in range(rows - 2, -1, step: -1) {
    base.at(j) = base.at(j + 1) + row-down.at(j + 1) + between.at(j) + row-up.at(j)
  }
  box(width: wide.at(0), height: base.at(0) + row-down.at(0), baseline: row-down.at(0), {
    for i in range(n) {
      let (k, name, c) = nodes.at(i)
      let (j, at) = (depth.at(i), x.at(i))
      place(dx: at + left.at(i), dy: base.at(j) - up.at(i), box(c))
      if k == none and open == "dots" {
        let dx = at + (wide.at(i) - dots-width) / 2
        place(dx: dx, dy: base.at(j + 1) - dots-up, box(dots))
      }
      if bar.at(i) != none {
        let (start, end) = bar.at(i)
        let y = base.at(j) - row-up.at(j) - between.at(j) / 2
        let dash = if k == none { "dashed" } else { none }
        place(dx: at + start, dy: y, line(length: end - start, stroke: (thickness: stroke, dash: dash)))
        if names.at(i) != none {
          let (w, h) = names.at(i)
          place(dx: at + end + name-gap, dy: y - h / 2, box(name))
        }
      }
    }
  })
}
