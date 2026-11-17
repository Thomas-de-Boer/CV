// Writes the project cards from assets/projects.json into a page as plain HTML,
// between <!-- projects:start --> and <!-- projects:end -->, so the projects are
// readable without JavaScript (search engines, AI crawlers). With JavaScript,
// assets/site.js replaces them on load, so keep the markup in step with
// projectCard() there. English text, the page's default language.
//
// Usage, from the repository root: node tools/render-projects.mjs [page.html]
// (default: index.html). No dependencies.
import { readFileSync, writeFileSync } from "node:fs";

const page = process.argv[2] ?? "index.html";
const GITHUB_USER = "Thomas-de-Boer";
const MAX_PROJECTS = 6; // same as assets/site.js

const escape = (text) =>
  String(text).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);

function card({ cover, kind, title, desc, tags = [], href, linkText, image }) {
  const coverHtml = image ? `<img alt="" loading="lazy" src="${escape(image)}">` : escape(cover);
  const kindHtml = kind ? `<span class="label">${escape(kind)}</span>` : "";
  const tagsHtml = tags.length
    ? `<div class="tags">${tags.map((tag) => `<span class="tag">${escape(tag)}</span>`).join("")}</div>`
    : "";
  return (
    `<article class="card project"><div class="project-cover">${coverHtml}</div>` +
    `<div class="project-body">${kindHtml}<h3 class="title">${escape(title)}</h3><p>${escape(desc)}</p>${tagsHtml}` +
    `<a class="project-link" href="${escape(href)}" rel="noopener">${escape(linkText)}</a></div></article>`
  );
}

const repos = JSON.parse(readFileSync("assets/projects.json", "utf8")).slice(0, MAX_PROJECTS);
const cards = repos.map((repo) =>
  card({
    cover: repo.name,
    kind: repo.language,
    title: repo.name,
    desc: repo.description || "No description yet.",
    tags: (repo.topics || []).slice(0, 4),
    href: repo.url,
    linkText: "View on GitHub →",
    image: repo.image,
  }),
);
cards.push(
  card({
    cover: "GitHub",
    title: "More on GitHub",
    desc: "All my public repositories are on my GitHub profile.",
    href: `https://github.com/${GITHUB_USER}`,
    linkText: "Open my GitHub profile →",
  }),
);

const html = readFileSync(page, "utf8");
const pattern = /<!-- projects:start -->[\s\S]*?<!-- projects:end -->/;
if (!pattern.test(html)) {
  console.error(`${page}: no <!-- projects:start --> ... <!-- projects:end --> markers found.`);
  process.exit(1);
}
const block = `<!-- projects:start -->\n    ${cards.join("\n    ")}\n    <!-- projects:end -->`;
writeFileSync(page, html.replace(pattern, () => block));
console.log(`Wrote ${repos.length} projects into ${page}`);
