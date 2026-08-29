// Shared specification document system for the fy ecosystem.
//
// The PDF output ships a single print palette (light); HTML theming lives in
// the fy-docs viewer stylesheet, not here.

#let fonts = (
  serif: ("Noto Serif SC", "SimSun", "Times New Roman"),
  sans: ("Noto Sans SC", "Microsoft YaHei", "Arial"),
  mono: ("Source Code Pro", "Cascadia Code", "DejaVu Sans Mono", "Consolas", "Courier New"),
)

#let palette = (
  paper: rgb("#ffffff"),
  ink: rgb("#243447"),
  heading: rgb("#172033"),
  muted: rgb("#64748b"),
  border: rgb("#cbd5e1"),
  accent: rgb("#2563eb"),
  code-bg: rgb("#f8fafc"),
  code-fg: rgb("#243447"),
  code-border: rgb("#e2e8f0"),
  inline-bg: rgb("#f1f5f9"),
  inline-fg: rgb("#0f172a"),
  syntax: auto,
  h1-fill: rgb("#1e3a5f"),
  h2-fill: rgb("#2c3e50"),
  h3-fill: rgb("#34495e"),
  chip-bg: rgb("#1e293b"),
  chip-fg: rgb("#ffffff"),
  title-fill: rgb("#0f172a"),
  meta-label: rgb("#475569"),
  meta-value: rgb("#1e293b"),
  note: (accent: rgb("#2563eb"), bg: rgb("#eff6ff"), border: rgb("#bfdbfe")),
  contract: (accent: rgb("#15803d"), bg: rgb("#f0fdf4"), border: rgb("#bbf7d0")),
  invariant: (accent: rgb("#a16207"), bg: rgb("#fefce8"), border: rgb("#fef08a")),
  example: (accent: rgb("#7e22ce"), bg: rgb("#faf5ff"), border: rgb("#e9d5ff")),
  badge-pending: (fg: rgb("#9a3412"), bg: rgb("#fff7ed"), border: rgb("#fdba74")),
  badge-done: (fg: rgb("#1e40af"), bg: rgb("#eff6ff"), border: rgb("#93c5fd")),
)

// Deprecated aliases kept for existing chapter sources.
#let colors = (
  ink: palette.ink,
  heading: palette.heading,
  muted: palette.muted,
  border: palette.border,
  accent: palette.accent,
  code-bg: palette.code-bg,
  inline-code-bg: palette.inline-bg,
)
#let font-serif = fonts.serif
#let font-sans = fonts.sans
#let font-mono = fonts.mono

/// Renders an ISO B5 specification book for an fy project.
#let project_book(
  title: "fy 规格说明书",
  subtitle: "契约驱动的软件规格与设计",
  version: "0.1.0",
  author: "fengyangsi",
  date: datetime.today().display("[year]-[month]-[day]"),
  methodology: "541 演进式契约驱动开发",
  body,
) = {
  set document(title: title, author: author)

  set page(
    paper: "iso-b5",
    fill: palette.paper,
    margin: (
      top: 25mm,
      bottom: 22mm,
      inside: 24mm,
      outside: 18mm,
    ),
    header: context {
      let page-number = counter(page).get().first()
      if page-number > 2 {
        let headings = query(selector(heading.where(level: 1)).before(here()))
        let current-title = if headings.len() > 0 {
          headings.last().body
        } else {
          title
        }

        if calc.even(page-number) {
          text(size: 8.5pt, fill: palette.muted, font: fonts.sans)[#title · #version]
        } else {
          align(right, text(size: 8.5pt, fill: palette.muted, font: fonts.sans, current-title))
        }
        v(1pt)
        line(length: 100%, stroke: 0.5pt + palette.border)
      }
    },
    footer: context {
      let page-number = counter(page).get().first()
      if page-number > 2 {
        let page-label = text(
          size: 9pt,
          weight: "medium",
          fill: palette.ink,
          font: fonts.mono,
          str(page-number),
        )
        if calc.even(page-number) {
          align(left, page-label)
        } else {
          align(right, page-label)
        }
      }
    },
  )

  set text(
    font: fonts.serif,
    size: 10pt,
    fill: palette.ink,
    lang: "zh",
    region: "cn",
  )
  set par(leading: 0.75em, justify: true, first-line-indent: 0em)

  set raw(theme: palette.syntax)
  show raw: set text(font: fonts.mono, size: 8.5pt, fill: palette.code-fg)
  show raw.where(block: true): content => block(
    width: 100%,
    fill: palette.code-bg,
    inset: (x: 10pt, y: 8pt),
    radius: 4pt,
    stroke: 0.5pt + palette.code-border,
    content,
  )
  show raw.where(block: false): content => box(
    fill: palette.inline-bg,
    inset: (x: 3pt, y: 1.5pt),
    radius: 2.5pt,
    baseline: 0%,
    text(fill: palette.inline-fg, content),
  )

  set heading(numbering: "1.1")
  show heading: content => block(
    above: 1.2em,
    below: 0.6em,
    text(font: fonts.sans, fill: palette.heading, weight: "bold", content),
  )
  show heading.where(level: 1): content => {
    pagebreak(weak: true)
    v(12pt)
    block(
      width: 100%,
      stroke: (bottom: 1.5pt + palette.accent),
      inset: (bottom: 8pt),
      text(size: 16pt, fill: palette.h1-fill, font: fonts.sans, content),
    )
    v(6pt)
  }
  show heading.where(level: 2): content => text(
    size: 12pt,
    fill: palette.h2-fill,
    content,
  )
  show heading.where(level: 3): content => text(
    size: 10.5pt,
    fill: palette.h3-fill,
    content,
  )

  align(center + horizon)[
    #v(-20pt)
    #rect(
      fill: palette.chip-bg,
      radius: 6pt,
      inset: (x: 16pt, y: 6pt),
    )[
      #text(size: 9pt, weight: "bold", fill: palette.chip-fg, font: fonts.mono)[
        PROJECT SPECIFICATION · ISO B5 EDITION
      ]
    ]

    #v(20pt)
    #text(size: 26pt, weight: "bold", fill: palette.title-fill, font: fonts.sans)[#title]
    #v(8pt)
    #text(size: 13pt, fill: palette.muted, font: fonts.serif)[#subtitle]
    #v(16pt)
    #line(length: 40%, stroke: 1.5pt + palette.accent)
    #v(40pt)
    #grid(
      columns: (auto, auto),
      gutter: 14pt,
      align: left,
      text(weight: "bold", fill: palette.meta-label, font: fonts.sans)[版本 (Version):],
      text(font: fonts.mono, fill: palette.meta-value)[#version],
      text(weight: "bold", fill: palette.meta-label, font: fonts.sans)[架构师 (Author):],
      text(fill: palette.meta-value)[#author],
      text(weight: "bold", fill: palette.meta-label, font: fonts.sans)[构建日期 (Date):],
      text(font: fonts.mono, fill: palette.meta-value)[#date],
      text(weight: "bold", fill: palette.meta-label, font: fonts.sans)[核心范式 (Methodology):],
      text(fill: palette.meta-value)[#methodology],
    )
  ]
  pagebreak()

  v(10pt)
  align(center)[
    #text(size: 16pt, weight: "bold", fill: palette.title-fill, font: fonts.sans)[目 录]
  ]
  v(12pt)
  outline(title: none, depth: 3, indent: 1.5em)
  pagebreak()

  body
}

/// Base component for semantic callouts. `kind` feeds the `fy-box fy-<kind>`
/// classes emitted under HTML export, so the viewer stylesheet can target a
/// box family without scraping typst's own structure.
#let callout(
  body,
  title: "提示",
  icon: none,
  kind: "note",
  accent: palette.note.accent,
  background: palette.note.bg,
  border: palette.note.border,
) = context {
  if target() == "html" {
    html.div(class: ("fy-box", "fy-" + kind))[
      #html.span(class: "fy-box-title")[#if icon != none { [#icon ] }#title]
      #body
    ]
  } else {
    block(
      width: 100%,
      fill: background,
      stroke: (left: 3.5pt + accent, rest: 0.5pt + border),
      inset: (x: 12pt, y: 9pt),
      radius: (right: 4pt),
      above: 10pt,
      below: 10pt,
    )[
      #text(weight: "bold", fill: accent, size: 9pt)[
        #if icon != none { [#icon ] }#title
      ]
      #v(4pt)
      #body
    ]
  }
}

#let contract(body) = callout(
  body,
  title: "强类型接口与规格契约 (Contract)",
  icon: "▣",
  kind: "contract",
  accent: palette.contract.accent,
  background: palette.contract.bg,
  border: palette.contract.border,
)

#let invariant(body) = callout(
  body,
  title: "核心不变性与安全约束 (Invariant)",
  icon: "◆",
  kind: "invariant",
  accent: palette.invariant.accent,
  background: palette.invariant.bg,
  border: palette.invariant.border,
)

#let logic-box(body) = callout(
  body,
  title: "形式逻辑与推理规则 (Logical Rules)",
  icon: "◇",
  kind: "logic",
)

#let proof-box(body) = callout(
  body,
  title: "证明策略与启发式搜索 (Proof Strategy)",
  icon: "◇",
  kind: "proof",
)

#let math-box(body) = callout(
  body,
  title: "数学推导与数值模型 (Mathematical Model)",
  icon: "◇",
  kind: "math",
)

#let geom-box(body) = callout(
  body,
  title: "几何结构与空间模型 (Geometry Model)",
  icon: "◇",
  kind: "geom",
)

#let axiom-box(body) = callout(
  body,
  title: "几何公理与推演法则 (Geometric Axiom)",
  icon: "◇",
  kind: "axiom",
)

#let motion-box(body) = callout(
  body,
  title: "运动学模型与时序法则 (Motion Model)",
  icon: "◇",
  kind: "motion",
)

#let example-box(body) = callout(
  body,
  title: "规格用例与状态验证 (Example & Verification)",
  icon: "●",
  kind: "example",
  accent: palette.example.accent,
  background: palette.example.bg,
  border: palette.example.border,
)

#let status-badge(status: "待确立", phase: "阶段 1") = {
  let pending = status.contains("待")
  let state = if pending { "pending" } else { "done" }
  let marker = if pending { "○" } else { "✓" }

  context {
    if target() == "html" {
      html.span(class: ("fy-badge", "fy-badge-" + state))[
        #marker 状态: #status | 阶段: #phase
      ]
    } else {
      let badge = if pending { palette.badge-pending } else { palette.badge-done }
      box(
        fill: badge.bg,
        stroke: 0.5pt + badge.border,
        inset: (x: 6pt, y: 2.5pt),
        radius: 3pt,
        baseline: 0%,
      )[
        #text(size: 8pt, weight: "bold", fill: badge.fg, font: fonts.mono)[
          #marker 状态: #status | 阶段: #phase
        ]
      ]
    }
  }
}

