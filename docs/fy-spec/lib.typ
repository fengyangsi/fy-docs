// Shared specification document system for the fy ecosystem.
//
// Language-agnostic & font-decoupled design system for technical specifications.
// The PDF output ships a single print palette (light); HTML theming lives in
// the fy-docs viewer stylesheet, not here.

#let default-fonts = (
  serif: (
    "Noto Serif SC",
    "Noto Serif CJK SC",
    "Source Han Serif SC",
    "Songti SC",
    "SimSun",
    "Times New Roman",
    "Linux Libertine",
  ),
  sans: (
    "Noto Sans SC",
    "Noto Sans CJK SC",
    "Source Han Sans SC",
    "PingFang SC",
    "Microsoft YaHei",
    "Arial",
    "Helvetica",
  ),
  mono: (
    "Source Code Pro",
    "Cascadia Code",
    "DejaVu Sans Mono",
    "Consolas",
    "Courier New",
  ),
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
#let font-serif = default-fonts.serif
#let font-sans = default-fonts.sans
#let font-mono = default-fonts.mono

/// Multi-language localization dictionary for specifications.
#let i18n-strings = (
  "en": (
    edition: "PROJECT SPECIFICATION · ISO B5 EDITION",
    version: "Version:",
    author: "Author:",
    date: "Date:",
    methodology: "Methodology:",
    toc: "Table of Contents",
    contract: "Interface Contract",
    invariant: "Core Invariant",
    example: "Specification Example",
    logic: "Logical Rules",
    proof: "Proof Strategy",
    math: "Mathematical Model",
    geom: "Geometry Model",
    axiom: "Geometric Axiom",
    motion: "Motion Model",
    status-label: "Status:",
    phase-label: "Phase:",
    status-pending: "Pending",
    status-done: "Established",
  ),
  "zh": (
    edition: "规格说明书 · ISO B5 典藏版",
    version: "版本:",
    author: "作者:",
    date: "构建日期:",
    methodology: "核心范式:",
    toc: "目 录",
    contract: "强类型接口与规格契约",
    invariant: "核心不变性与安全约束",
    example: "规格用例与状态验证",
    logic: "形式逻辑与推理规则",
    proof: "证明策略与启发式搜索",
    math: "数学推导与数值模型",
    geom: "几何结构与空间模型",
    axiom: "几何公理与推演法则",
    motion: "运动学模型与时序法则",
    status-label: "状态:",
    phase-label: "阶段:",
    status-pending: "待确立",
    status-done: "已确立",
  ),
  "ja": (
    edition: "仕様書 · ISO B5 版",
    version: "バージョン:",
    author: "作成者:",
    date: "作成日:",
    methodology: "方法論:",
    toc: "目 次",
    contract: "インターフェース契約",
    invariant: "不変条件",
    example: "仕様例と検証",
    logic: "論理規則",
    proof: "証明戦略",
    math: "数学モデル",
    geom: "幾何学モデル",
    axiom: "幾何公理",
    motion: "運動学モデル",
    status-label: "状態:",
    phase-label: "フェーズ:",
    status-pending: "未確立",
    status-done: "確立済",
  ),
  "de": (
    edition: "PROJEKTSPEZIFIKATION · ISO B5 AUSGABE",
    version: "Version:",
    author: "Autor:",
    date: "Datum:",
    methodology: "Methodik:",
    toc: "Inhaltsverzeichnis",
    contract: "Schnittstellenvertrag",
    invariant: "Kerninvariante",
    example: "Spezifikationsbeispiel",
    logic: "Logische Regeln",
    proof: "Beweisstrategie",
    math: "Mathematisches Modell",
    geom: "Geometriemodell",
    axiom: "Geometrisches Axiom",
    motion: "Bewegungsmodell",
    status-label: "Status:",
    phase-label: "Phase:",
    status-pending: "Ausstehend",
    status-done: "Etabliert",
  ),
  "fr": (
    edition: "SPÉCIFICATION DU PROJET · ÉDITION ISO B5",
    version: "Version :",
    author: "Auteur :",
    date: "Date :",
    methodology: "Méthodologie :",
    toc: "Table des matières",
    contract: "Contrat d'interface",
    invariant: "Invariant central",
    example: "Exemple de spécification",
    logic: "Règles logiques",
    proof: "Stratégie de preuve",
    math: "Modèle mathématique",
    geom: "Modèle géométrique",
    axiom: "Axiome géométrique",
    motion: "Modèle cinématique",
    status-label: "Statut :",
    phase-label: "Phase :",
    status-pending: "En attente",
    status-done: "Établi",
  ),
)

/// Resolves a localized string key from the dictionary with safe fallback.
#let resolve-i18n(key, lang: auto) = {
  let cur-lang = if lang != auto { lang } else { text.lang }
  let base-lang = if cur-lang != none and cur-lang.contains("-") { cur-lang.split("-").at(0) } else { cur-lang }

  if cur-lang != none and cur-lang in i18n-strings and key in i18n-strings.at(cur-lang) {
    i18n-strings.at(cur-lang).at(key)
  } else if base-lang != none and base-lang in i18n-strings and key in i18n-strings.at(base-lang) {
    i18n-strings.at(base-lang).at(key)
  } else if key in i18n-strings.at("en") {
    i18n-strings.at("en").at(key)
  } else {
    key
  }
}

/// Horizontally centered content that survives HTML export.
#let centered(body) = context {
  if target() == "html" {
    html.div(style: "text-align: center", body)
  } else {
    align(center, body)
  }
}

/// Renders an ISO B5 specification book for a project.
///
/// `lang` declares the document's content language: Typst typesets from it, and
/// fy-docs reports the same tag as the generated page's `<html lang>` and picks
/// its toolbar labels from it.
#let project_book(
  title: "Project Specification",
  subtitle: none,
  version: "0.1.0",
  author: none,
  date: datetime.today().display("[year]-[month]-[day]"),
  lang: "en",
  region: none,
  fonts: (:),
  methodology: none,
  body,
) = {
  let (parsed-lang, parsed-region) = if lang.contains("-") {
    let parts = lang.split("-")
    (parts.at(0), if region == none { parts.at(1) } else { region })
  } else {
    (lang, region)
  }

  let active-fonts = (
    serif: if "serif" in fonts { fonts.serif } else { default-fonts.serif },
    sans: if "sans" in fonts { fonts.sans } else { default-fonts.sans },
    mono: if "mono" in fonts { fonts.mono } else { default-fonts.mono },
  )

  let labels = (
    edition: resolve-i18n("edition", lang: parsed-lang),
    version: resolve-i18n("version", lang: parsed-lang),
    author: resolve-i18n("author", lang: parsed-lang),
    date: resolve-i18n("date", lang: parsed-lang),
    methodology: resolve-i18n("methodology", lang: parsed-lang),
    toc: resolve-i18n("toc", lang: parsed-lang),
  )

  set document(title: title, author: if author != none { author } else { () })

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
          text(size: 8.5pt, fill: palette.muted, font: active-fonts.sans)[#title · #version]
        } else {
          align(right, text(size: 8.5pt, fill: palette.muted, font: active-fonts.sans, current-title))
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
          font: active-fonts.mono,
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
    font: active-fonts.serif,
    size: 10pt,
    fill: palette.ink,
    lang: parsed-lang,
    region: parsed-region,
  )
  set par(leading: 0.75em, justify: true, first-line-indent: 0em)

  set raw(theme: palette.syntax)
  show raw: set text(font: active-fonts.mono, size: 8.5pt, fill: palette.code-fg)
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
    text(font: active-fonts.sans, fill: palette.heading, weight: "bold", content),
  )
  show heading.where(level: 1): content => {
    pagebreak(weak: true)
    v(12pt)
    block(
      width: 100%,
      stroke: (bottom: 1.5pt + palette.accent),
      inset: (bottom: 8pt),
      text(size: 16pt, fill: palette.h1-fill, font: active-fonts.sans, content),
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

  // Assemble metadata grid items dynamically
  let meta-grid-items = (
    text(weight: "bold", fill: palette.meta-label, font: active-fonts.sans)[#labels.version],
    text(font: active-fonts.mono, fill: palette.meta-value)[#version],
  )
  if author != none {
    meta-grid-items.push(text(weight: "bold", fill: palette.meta-label, font: active-fonts.sans)[#labels.author])
    meta-grid-items.push(text(fill: palette.meta-value)[#author])
  }
  meta-grid-items.push(text(weight: "bold", fill: palette.meta-label, font: active-fonts.sans)[#labels.date])
  meta-grid-items.push(text(font: active-fonts.mono, fill: palette.meta-value)[#date])
  if methodology != none {
    meta-grid-items.push(text(weight: "bold", fill: palette.meta-label, font: active-fonts.sans)[#labels.methodology])
    meta-grid-items.push(text(fill: palette.meta-value)[#methodology])
  }

  let cover = [
    #v(-20pt)
    #rect(
      fill: palette.chip-bg,
      radius: 6pt,
      inset: (x: 16pt, y: 6pt),
    )[
      #text(size: 9pt, weight: "bold", fill: palette.chip-fg, font: active-fonts.mono)[
        #labels.edition
      ]
    ]

    #v(20pt)
    #text(size: 26pt, weight: "bold", fill: palette.title-fill, font: active-fonts.sans)[#title]
    #if subtitle != none [
      #v(8pt)
      #text(size: 13pt, fill: palette.muted, font: active-fonts.serif)[#subtitle]
    ]
    #v(16pt)
    #line(length: 40%, stroke: 1.5pt + palette.accent)
    #v(32pt)
    #grid(
      columns: (auto, auto),
      gutter: 14pt,
      align: left,
      ..meta-grid-items
    )
  ]

  context {
    if target() == "html" {
      html.div(class: "fy-cover", style: "text-align: center")[
        #html.span(class: "fy-cover-chip")[
          #text(size: 9pt, weight: "bold", font: active-fonts.mono)[
            #labels.edition
          ]
        ]
        #parbreak()
        #text(size: 26pt, weight: "bold", fill: palette.title-fill, font: active-fonts.sans)[#title]
        #if subtitle != none [
          #parbreak()
          #text(size: 13pt, fill: palette.muted, font: active-fonts.serif)[#subtitle]
        ]
        #parbreak()
        #html.elem("dl", attrs: (class: "fy-cover-meta"))[
          #html.elem("dt")[#labels.version]
          #html.elem("dd")[#text(font: active-fonts.mono)[#version]]
          #if author != none [
            #html.elem("dt")[#labels.author]
            #html.elem("dd")[#author]
          ]
          #html.elem("dt")[#labels.date]
          #html.elem("dd")[#text(font: active-fonts.mono)[#date]]
          #if methodology != none [
            #html.elem("dt")[#labels.methodology]
            #html.elem("dd")[#methodology]
          ]
        ]
      ]
    } else {
      align(center + horizon, cover)
    }
  }
  pagebreak()

  v(10pt)
  centered[
    #text(size: 16pt, weight: "bold", fill: palette.title-fill, font: active-fonts.sans)[#labels.toc]
  ]
  v(12pt)
  outline(title: none, depth: 3, indent: 1.5em)
  pagebreak()

  body
}

/// Base component for semantic callouts.
#let callout(
  body,
  title: "Note",
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

#let contract(body, title: auto) = context {
  let effective-title = if title == auto { resolve-i18n("contract") } else { title }
  callout(
    body,
    title: effective-title,
    icon: "▣",
    kind: "contract",
    accent: palette.contract.accent,
    background: palette.contract.bg,
    border: palette.contract.border,
  )
}

#let invariant(body, title: auto) = context {
  let effective-title = if title == auto { resolve-i18n("invariant") } else { title }
  callout(
    body,
    title: effective-title,
    icon: "◆",
    kind: "invariant",
    accent: palette.invariant.accent,
    background: palette.invariant.bg,
    border: palette.invariant.border,
  )
}

#let logic-box(body, title: auto) = context {
  let effective-title = if title == auto { resolve-i18n("logic") } else { title }
  callout(
    body,
    title: effective-title,
    icon: "◇",
    kind: "logic",
  )
}

#let proof-box(body, title: auto) = context {
  let effective-title = if title == auto { resolve-i18n("proof") } else { title }
  callout(
    body,
    title: effective-title,
    icon: "◇",
    kind: "proof",
  )
}

#let math-box(body, title: auto) = context {
  let effective-title = if title == auto { resolve-i18n("math") } else { title }
  callout(
    body,
    title: effective-title,
    icon: "◇",
    kind: "math",
  )
}

#let geom-box(body, title: auto) = context {
  let effective-title = if title == auto { resolve-i18n("geom") } else { title }
  callout(
    body,
    title: effective-title,
    icon: "◇",
    kind: "geom",
  )
}

#let axiom-box(body, title: auto) = context {
  let effective-title = if title == auto { resolve-i18n("axiom") } else { title }
  callout(
    body,
    title: effective-title,
    icon: "◇",
    kind: "axiom",
  )
}

#let motion-box(body, title: auto) = context {
  let effective-title = if title == auto { resolve-i18n("motion") } else { title }
  callout(
    body,
    title: effective-title,
    icon: "◇",
    kind: "motion",
  )
}

#let example-box(body, title: auto) = context {
  let effective-title = if title == auto { resolve-i18n("example") } else { title }
  callout(
    body,
    title: effective-title,
    icon: "●",
    kind: "example",
    accent: palette.example.accent,
    background: palette.example.bg,
    border: palette.example.border,
  )
}

#let status-badge(status: auto, phase: auto) = context {
  let def-status = if status == auto { resolve-i18n("status-pending") } else { status }
  let def-phase = if phase == auto { "1" } else { phase }
  let pending = def-status.contains("待") or def-status.contains("pending") or def-status.contains("WIP") or def-status.contains("未") or def-status.contains("Ausstehend") or def-status.contains("attente")
  let state = if pending { "pending" } else { "done" }
  let marker = if pending { "○" } else { "✓" }
  let status-lbl = resolve-i18n("status-label")
  let phase-lbl = resolve-i18n("phase-label")

  if target() == "html" {
    html.span(class: ("fy-badge", "fy-badge-" + state))[
      #marker #status-lbl #def-status | #phase-lbl #def-phase
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
      #text(size: 8pt, weight: "bold", fill: badge.fg, font: default-fonts.mono)[
        #marker #status-lbl #def-status | #phase-lbl #def-phase
      ]
    ]
  }
}
