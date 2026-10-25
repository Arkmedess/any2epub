//! Folha de estilos editorial tipográfica (`OEBPS/style.css`) compatível com motores e-ink e W3C EPUB 3.3.

/// Retorna a folha de estilos CSS editorial completa.
pub fn build_editorial_css() -> &'static str {
    r#"@charset "UTF-8";
body {
  font-family: Georgia, "Bookerly", "Literata", serif;
  line-height: 1.55;
  margin: 4% 5%;
  color: #1a1a1a;
}
body.cover-body {
  margin: 0;
  padding: 0;
  text-align: center;
}
div.cover-container {
  margin: 0;
  padding: 0;
  height: 100vh;
}
img.cover-image {
  max-width: 100%;
  max-height: 98vh;
  object-fit: contain;
}
h1.chapter-title {
  text-align: right;
  font-size: 1.55em;
  font-weight: bold;
  border-bottom: 2px solid #1793d1;
  padding-bottom: 0.4em;
  margin-top: 1.5em;
  margin-bottom: 1.4em;
}
h2 {
  font-size: 1.25em;
  margin-top: 1.4em;
  margin-bottom: 0.5em;
}
h3 {
  font-size: 1.08em;
  margin-top: 1.1em;
  margin-bottom: 0.4em;
}
h4 {
  font-size: 0.98em;
  font-style: italic;
  margin-top: 0.9em;
  margin-bottom: 0.35em;
}
span.sec-num {
  font-family: "Courier New", monospace;
  font-weight: bold;
  color: #555;
  margin-right: 0.35em;
}
ol.toc-sublist {
  list-style-type: none;
  padding-left: 1.25em;
  margin-top: 0.2em;
  margin-bottom: 0.35em;
}
p {
  text-indent: 1.2em;
  margin-top: 0;
  margin-bottom: 0.45em;
  text-align: justify;
}
p.first-para, p.dropcap-para, p.meta-line {
  text-indent: 0;
}
span.dropcap {
  float: left;
  font-size: 3.1em;
  line-height: 0.85;
  padding-top: 0.04em;
  padding-right: 0.1em;
  font-weight: bold;
}
p.dialogue {
  text-indent: 1.2em;
  margin-top: 0;
  margin-bottom: 0.3em;
}
pre.code-block {
  font-family: "Courier New", monospace;
  font-size: 0.84em;
  background-color: #f5f5f5;
  border-left: 3px solid #1793d1;
  padding: 0.6em 0.8em;
  margin: 0.8em 0;
  white-space: pre-wrap;
  word-wrap: break-word;
}
code {
  font-family: "Courier New", monospace;
  font-size: 0.9em;
}
aside.callout {
  padding: 0.6em 0.9em;
  margin: 0.8em 0;
  border-left: 4px solid #555;
  background-color: #f8f9fa;
}
aside.callout-note { border-left-color: #1793d1; }
aside.callout-tip { border-left-color: #2e7d32; }
aside.callout-warning { border-left-color: #c62828; background-color: #fff8f8; }
table.wikitable {
  width: 100%;
  border-collapse: collapse;
  margin: 1em 0;
  font-size: 0.88em;
}
table.wikitable th, table.wikitable td {
  border: 1px solid #aaa;
  padding: 0.4em 0.6em;
  text-align: left;
}
table.wikitable th {
  background-color: #eaeff2;
  font-weight: bold;
}
figure.book-figure {
  margin: 1.2em auto;
  text-align: center;
  page-break-inside: avoid;
}
figure.book-figure img {
  max-width: 96%;
  height: auto;
}
p.fig-caption {
  text-indent: 0;
  font-size: 0.88em;
  font-style: italic;
  text-align: center;
  margin-bottom: 1.1em;
}
aside[epub|type~="footnote"] {
  font-size: 0.85em;
  line-height: 1.35;
  margin-bottom: 0.6em;
}
"#
}
