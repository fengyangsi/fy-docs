//! Page template and bundled assets for the generated static page.

use crate::project::LanguageTarget;

const DOC_TEMPLATE: &str = include_str!("../assets/doc.html");
pub(crate) const BASE_CSS: &str = include_str!("../assets/base.css");
pub(crate) const VIEWER_JS: &str = include_str!("../assets/viewer.js");
/// Live-reload client, shipped with every build: it subscribes to the
/// dev server's `/events` stream and stays silent on static `file://` pages.
pub(crate) const LIVE_JS: &str = include_str!("../assets/live.js");

pub(crate) struct UiText {
    pub(crate) language: &'static str,
    pub(crate) sidebar_toggle: &'static str,
    pub(crate) theme: &'static str,
    pub(crate) system_theme: &'static str,
    pub(crate) search: &'static str,
    pub(crate) search_document: &'static str,
    pub(crate) search_placeholder: &'static str,
    pub(crate) print: &'static str,
    pub(crate) table_of_contents: &'static str,
    pub(crate) github: &'static str,
    pub(crate) language_label: &'static str,
    pub(crate) compile_failed: &'static str,
    pub(crate) compile_failed_detail: &'static str,
    pub(crate) compile_failed_hint: &'static str,
}

/// Chooses the UI language from the document's declared language when a
/// target is known; the fallback scans the body for CJK ideographs (used by
/// default-language projects whose `lang` field is empty).
pub(crate) fn ui_text(lang_hint: Option<&str>, body: &str) -> UiText {
    let chinese = match lang_hint.filter(|hint| !hint.is_empty()) {
        Some(lang) => lang.to_lowercase().starts_with("zh"),
        None => body
            .chars()
            .any(|character| ('\u{4e00}'..='\u{9fff}').contains(&character)),
    };
    if chinese {
        UiText {
            language: "zh-CN",
            sidebar_toggle: "目录侧栏",
            theme: "主题",
            system_theme: "跟随系统",
            search: "搜索",
            search_document: "搜索文档",
            search_placeholder: "输入关键词",
            print: "打印当前章节",
            table_of_contents: "目录",
            github: "GitHub 仓库",
            language_label: "切换语言",
            compile_failed: "编译失败",
            compile_failed_detail: "typst 编译未能通过，输出如下：",
            compile_failed_hint: "修正源码并保存后，本页会在下次编译完成时自动更新。",
        }
    } else {
        UiText {
            language: "en",
            sidebar_toggle: "Toggle sidebar",
            theme: "Theme",
            system_theme: "System preference",
            search: "Search",
            search_document: "Search documentation",
            search_placeholder: "Enter keywords",
            print: "Print current chapter",
            table_of_contents: "Table of contents",
            github: "GitHub repository",
            language_label: "Language",
            compile_failed: "Compilation failed",
            compile_failed_detail: "typst compilation failed with the following output:",
            compile_failed_hint: "Fix the source and save; this page will update automatically on the next successful build.",
        }
    }
}

/// The language of the page *content*, written on the document root.
///
/// Chrome strings and content language are two separate axes: toolbar labels
/// exist only in English and Chinese and `viewer.js` picks between them from
/// the root `lang` prefix, but the root attribute itself must describe the
/// text. A Portuguese document therefore says `pt-BR` while wearing English
/// chrome, rather than claiming to be English.
fn content_lang(target: Option<&LanguageTarget>, ui: &UiText) -> String {
    match target.map(|t| crate::project::normalize_lang(&t.lang)) {
        Some(tag) if !tag.is_empty() => crate::project::format_lang(&tag),
        _ => ui.language.to_owned(),
    }
}

/// Renders the generated page. The Typst body is already trimmed; GitHub is
/// linked only when the package declares that repository.
pub(crate) fn doc_page(
    title: &str,
    name: &str,
    repository: Option<&str>,
    body: &str,
    current_target: Option<&LanguageTarget>,
    all_targets: &[LanguageTarget],
) -> String {
    let ui = ui_text(current_target.map(|t| t.lang.as_str()), body);
    let github_link = repository.map_or_else(String::new, |url| {
        format!(
            r#"<a class="fy-github-link" href="{}" title="{}" aria-label="{}"><svg width="17" height="17" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 2a10 10 0 0 0-3.2 19.5c.5.1.7-.2.7-.5v-1.8c-2.8.6-3.4-1.2-3.4-1.2-.5-1.2-1.1-1.5-1.1-1.5-.9-.6.1-.6.1-.6 1 .1 1.5 1 1.5 1 .9 1.5 2.4 1.1 3 .8.1-.6.4-1.1.7-1.3-2.2-.2-4.6-1.1-4.6-5a3.9 3.9 0 0 1 1-2.7c-.1-.3-.4-1.3.1-2.7 0 0 .8-.3 2.8 1.1a9.7 9.7 0 0 1 5 0c2-1.4 2.8-1.1 2.8-1.1.5 1.4.2 2.4.1 2.7a3.9 3.9 0 0 1 1 2.7c0 3.9-2.4 4.7-4.6 5 .4.3.7 1 .7 1.9V21c0 .3.2.6.7.5A10 10 0 0 0 12 2Z"/></svg></a>"#,
            escape_attribute(url),
            ui.github,
            ui.github,
        )
    });

    let lang_menu = render_lang_menu(current_target, all_targets, ui.language_label);

    DOC_TEMPLATE
        .replace("{{TITLE}}", &escape(title))
        .replace("{{NAME}}", &escape(name))
        .replace("{{LANG}}", &content_lang(current_target, &ui))
        .replace("{{SIDEBAR_TOGGLE}}", ui.sidebar_toggle)
        .replace("{{THEME}}", ui.theme)
        .replace("{{SYSTEM_THEME}}", ui.system_theme)
        .replace("{{SEARCH}}", ui.search)
        .replace("{{SEARCH_DOCUMENT}}", ui.search_document)
        .replace("{{SEARCH_PLACEHOLDER}}", ui.search_placeholder)
        .replace("{{PRINT}}", ui.print)
        .replace("{{TABLE_OF_CONTENTS}}", ui.table_of_contents)
        .replace("{{GITHUB_LINK}}", &github_link)
        .replace("{{LANG_MENU}}", &lang_menu)
        .replace("{{BODY}}", body)
}

fn render_lang_menu(
    current_target: Option<&LanguageTarget>,
    all_targets: &[LanguageTarget],
    label: &str,
) -> String {
    // Filter distinct language targets (excluding empty default duplicates)
    let distinct_targets: Vec<&LanguageTarget> =
        all_targets.iter().filter(|t| !t.lang.is_empty()).collect();

    if distinct_targets.len() <= 1 {
        return String::new();
    }

    let mut items = String::new();
    let current_lang = current_target.map(|t| t.lang.as_str()).unwrap_or("");
    for target in &distinct_targets {
        let active = if current_lang.is_empty() {
            target.lang == distinct_targets[0].lang
        } else {
            target.lang.eq_ignore_ascii_case(current_lang)
        };
        items.push_str(&format!(
            r#"<a href="{}" role="menuitem" class="fy-lang-item" aria-checked="{}">{}</a>"#,
            escape_attribute(&target.html_file_name),
            if active { "true" } else { "false" },
            escape(&target.display_name)
        ));
    }

    format!(
        r#"<div class="fy-theme-wrap fy-lang-wrap">
        <button id="fy-lang-toggle" class="fy-icon-btn" aria-expanded="false" aria-haspopup="true" title="{label}">
          <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="10"/>
            <path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"/>
            <path d="M2 12h20"/>
          </svg>
        </button>
        <div id="fy-lang-menu" class="fy-theme-menu fy-lang-menu" role="menu" aria-label="{label}" hidden>
          {items}
        </div>
      </div>"#
    )
}

pub(crate) fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attribute(text: &str) -> String {
    escape(text).replace('"', "&quot;")
}

/// Escapes a value for a string literal inside an HTML `<script>` block.
/// HTML metacharacters are escaped first so a value can never break out of
/// the script element; backslashes, the quote character, and line/control
/// characters are then escaped so the literal stays valid JavaScript.
fn escape_script_string(text: &str, quote: char) -> String {
    let mut out = String::with_capacity(text.len());
    for c in escape(text).chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{2028}' | '\u{2029}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out
}

/// Escapes a value for a double-quoted JSON string in a script block.
fn escape_json_string(text: &str) -> String {
    escape_script_string(text, '"')
}

/// Escapes a value for a single-quoted JavaScript string literal.
fn escape_js_string(text: &str) -> String {
    escape_script_string(text, '\'')
}

/// Renders a tiny (~500B) client-side language routing landing page with dynamic matching.
pub(crate) fn redirect_page(all_targets: &[LanguageTarget]) -> String {
    let distinct: Vec<&LanguageTarget> =
        all_targets.iter().filter(|t| !t.lang.is_empty()).collect();

    let mut map_entries = Vec::new();
    for t in &distinct {
        let lang_lower = t.lang.to_lowercase();
        let file = &t.html_file_name;
        map_entries.push(format!(
            r#""{}":"{}""#,
            escape_json_string(&lang_lower),
            escape_json_string(file)
        ));
        if let Some((base, _)) = lang_lower.split_once('-') {
            if !base.is_empty() {
                map_entries.push(format!(
                    r#""{}":"{}""#,
                    escape_json_string(base),
                    escape_json_string(file)
                ));
            }
        }
    }
    let map_json = format!("{{{}}}", map_entries.join(","));

    let default_target = distinct
        .iter()
        .find(|t| t.lang.eq_ignore_ascii_case("en") || t.lang.to_lowercase().starts_with("en-"))
        .or_else(|| {
            distinct
                .iter()
                .find(|t| t.lang.eq_ignore_ascii_case("zh-cn") || t.lang.eq_ignore_ascii_case("zh"))
        })
        .or_else(|| distinct.first())
        .map(|t| t.html_file_name.as_str())
        .unwrap_or("index_en.html");

    let mut links = String::new();
    for (i, t) in distinct.iter().enumerate() {
        if i > 0 {
            links.push_str(" | ");
        }
        links.push_str(&format!(
            r#"<a href="{}">{}</a>"#,
            escape_attribute(&t.html_file_name),
            escape(&t.display_name)
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Redirecting...</title>
<script>
(function () {{
  var stored = null;
  try {{ stored = localStorage.getItem('fydocs-lang'); }} catch (_) {{}}
  if (stored) {{ location.replace(stored); return; }}
  var map = {map_json};
  var userLangs = (navigator.languages && navigator.languages.length) ? navigator.languages : [navigator.language || ''];
  for (var i = 0; i < userLangs.length; i++) {{
    var l = (userLangs[i] || '').toLowerCase();
    if (map[l]) {{ location.replace(map[l]); return; }}
    var base = l.split('-')[0];
    if (map[base]) {{ location.replace(map[base]); return; }}
  }}
  location.replace('{}');
}})();
</script>
<noscript><meta http-equiv="refresh" content="0; url={}"></noscript>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; text-align: center; padding: 40px;">
  <p>Redirecting / 正在跳转：{links}</p>
</body>
</html>
"#,
        escape_js_string(default_target),
        escape_attribute(default_target)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_all_tokens() {
        let page = doc_page(
            "T & <T>",
            "fy-x",
            Some("https://github.com/fengyangsi/fy-docs"),
            "<h1>hi</h1>",
            None,
            &[],
        );
        assert!(page.contains("<title>T &amp; &lt;T&gt; · fy-docs</title>"));
        assert!(page.contains("<html lang=\"en\">"));
        assert!(page.contains("title=\"Theme\""));
        assert!(page.contains("https://github.com/fengyangsi/fy-docs"));
        assert!(page.contains("<h1>hi</h1>"));
        assert!(page.contains("fy-theme-toggle"));
        assert!(page.contains("fy-search-toggle"));
        assert!(page.contains("fy-sidebar-resize"));
        assert!(page.contains("fy-docs.js"));
        assert!(page.contains("live.js"));
        assert!(!page.contains("{{"));
    }

    #[test]
    fn omits_github_link_without_a_repository() {
        let page = doc_page("T", "fy-x", None, "<h1>hi</h1>", None, &[]);
        assert!(!page.contains("fy-github-link"));
    }

    #[test]
    fn localizes_controls_for_chinese_documents() {
        let page = doc_page("中文文档", "fy-x", None, "<h1>内容</h1>", None, &[]);
        assert!(page.contains("<html lang=\"zh-CN\">"));
    }

    #[test]
    fn page_lang_describes_content_not_chrome() {
        let pt = LanguageTarget {
            lang: "pt_BR".to_owned(),
            display_name: "pt-BR".to_owned(),
            entry: std::path::PathBuf::from("docs/pt_BR/main.typ"),
            html_file_name: "index_pt_BR.html".to_owned(),
            pdf_file_name: "x.pdf".to_owned(),
        };
        // English toolbar labels must not relabel a Portuguese document.
        let page = doc_page("Doc", "fy-x", None, "<h1>Olá</h1>", Some(&pt), &[]);
        assert!(page.contains(r#"<html lang="pt-BR">"#), "{page}");

        // A tagless default target keeps inferring from the body.
        let zh = doc_page("T", "fy-x", None, "<h1>内容</h1>", None, &[]);
        assert!(zh.contains(r#"<html lang="zh-CN">"#), "{zh}");
    }

    #[test]
    fn lang_hint_overrides_body_detection() {
        let zh_target = LanguageTarget {
            lang: "zh-CN".to_owned(),
            display_name: "简体中文".to_owned(),
            entry: std::path::PathBuf::from("docs/zh-CN/main.typ"),
            html_file_name: "index_zh-CN.html".to_owned(),
            pdf_file_name: "x.pdf".to_owned(),
        };
        // An English body under a zh-CN target still gets Chinese controls.
        let page = doc_page("Doc", "fy-x", None, "<h1>hello</h1>", Some(&zh_target), &[]);
        assert!(page.contains("<html lang=\"zh-CN\">"));

        // A Japanese target must not be mistaken for Chinese by scanning.
        let ja_target = LanguageTarget {
            lang: "ja".to_owned(),
            display_name: "日本語".to_owned(),
            entry: std::path::PathBuf::from("docs/ja/main.typ"),
            html_file_name: "index_ja.html".to_owned(),
            pdf_file_name: "x.pdf".to_owned(),
        };
        let page = doc_page(
            "文書",
            "fy-x",
            None,
            "<h1>こんにちは</h1>",
            Some(&ja_target),
            &[],
        );
        // The root tag names the document; only the chrome falls back to English.
        assert!(page.contains("<html lang=\"ja\">"), "{page}");
        assert!(page.contains("title=\"Theme\""), "{page}");
        assert!(!page.contains("主题"), "{page}");
    }

    #[test]
    fn script_strings_escape_breakout_sequences() {
        assert_eq!(escape_json_string(r#"a"b\c"#), r#"a\"b\\c"#);
        assert_eq!(escape_json_string("a\nb\u{2028}c"), "a\\nb\\u2028c");
        assert_eq!(escape_json_string("a</script>b"), "a&lt;/script&gt;b");
        assert_eq!(escape_json_string("plain"), "plain");
        assert_eq!(escape_js_string("it's"), "it\\'s");
        assert_eq!(escape_js_string(r"a\b"), "a\\\\b");
        assert_eq!(escape_js_string("a</script>b"), "a&lt;/script&gt;b");
    }

    #[test]
    fn redirect_page_generates_dynamic_json_map() {
        let targets = vec![
            LanguageTarget {
                lang: "zh-CN".to_owned(),
                display_name: "简体中文".to_owned(),
                entry: std::path::PathBuf::from("docs/zh-CN/main.typ"),
                html_file_name: "index_zh-CN.html".to_owned(),
                pdf_file_name: "fy-x_v0.1.0_zh-CN_specification.pdf".to_owned(),
            },
            LanguageTarget {
                lang: "ja".to_owned(),
                display_name: "日本語".to_owned(),
                entry: std::path::PathBuf::from("docs/ja/main.typ"),
                html_file_name: "index_ja.html".to_owned(),
                pdf_file_name: "fy-x_v0.1.0_ja_specification.pdf".to_owned(),
            },
            LanguageTarget {
                lang: "en".to_owned(),
                display_name: "English".to_owned(),
                entry: std::path::PathBuf::from("docs/en/main.typ"),
                html_file_name: "index_en.html".to_owned(),
                pdf_file_name: "fy-x_v0.1.0_en_specification.pdf".to_owned(),
            },
        ];
        let html = redirect_page(&targets);
        assert!(html.contains(r#""zh-cn":"index_zh-CN.html""#));
        assert!(html.contains(r#""zh":"index_zh-CN.html""#));
        assert!(html.contains(r#""ja":"index_ja.html""#));
        assert!(html.contains(r#""en":"index_en.html""#));
        assert!(html.contains(r#"<a href="index_zh-CN.html">简体中文</a>"#));
        assert!(html.contains(r#"<a href="index_ja.html">日本語</a>"#));
        assert!(html.contains("location.replace('index_en.html')"));
    }

    #[test]
    fn landing_page_stays_lightweight() {
        let target = |lang: &str| LanguageTarget {
            lang: lang.to_owned(),
            display_name: lang.to_owned(),
            entry: std::path::PathBuf::from(format!("docs/{lang}/main.typ")),
            html_file_name: format!("index_{lang}.html"),
            pdf_file_name: format!("x_{lang}.pdf"),
        };
        let targets: Vec<LanguageTarget> = ["en", "zh-CN", "ja", "de", "fr"]
            .iter()
            .map(|l| target(l))
            .collect();

        let page = redirect_page(&targets);
        // The compiler chapter's invariant: routing script and language list
        // only, so five languages stay well under 1.4KB.
        assert!(
            page.len() < 1434,
            "landing page grew to {} bytes",
            page.len()
        );
        assert!(
            !page.contains("<h1") && !page.contains("fy-docs.js"),
            "the landing page must never carry document body or viewer assets"
        );
    }
}
