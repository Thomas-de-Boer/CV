// Language switch, theme switch and the projects slider. No framework.
(function () {
  "use strict";

  // The last project card always links to this GitHub profile.
  var GITHUB_USER = "Thomas-de-Boer";
  var MAX_PROJECTS = 6;

  var STRINGS = {
    en: {
      navProjects: "Projects", navNN: "Neural network", navWork: "Work & school", navMore: "More",
      langLabel: "Language", darkLabel: "Dark mode",
      badge: "Available for final internship · from August 2027",
      intro: "Second-year Software Development student at Nova College. I like the parts you don’t see: backends, databases and how systems fit together.",
      cta1: "See my projects", cta2: "Try the neural network",
      langsTitle: "Languages I work in", focus: "Trying a bit of everything right now; backend is where I want to go.",
      projTitle: "Things I’ve built", project: "project", projects: "projects",
      prev: "Previous projects", next: "Next projects", viewGh: "View on GitHub →",
      noDesc: "No description yet.", allTitle: "More on GitHub",
      allDesc: "All my public repositories are on my GitHub profile.", allLink: "Open my GitHub profile →",
      nnTitle: "Draw a number. Watch it think.",
      nnBody: "A neural network built from scratch, running in your browser. Draw a number on the pad and see each layer light up before it makes its guess.",
      noLibs: "No ML libraries", openDemo: "Open full demo",
      demoLabel: "Live demo", clear: "Clear", pad: "Drawing pad", input: "Draw a number here · 0–9", guess: "Guess",
      demoError: "The demo could not load here. Open the full demo instead.",
      wsTitle: "Where I’m at",
      wsBody: "I’m heading towards backend development, databases and systems architecture, with a long-term interest in AI and machine learning.",
      school: "School", work: "Work", now: "Now",
      nova: "Nova College · second year", prevSchool: "Dali College", level: "Mavo",
      vomarRole: "Part-time · Shelf stocker / Cashier", motiRole: "Internship (blokstage) · working on a planning tool", nextLabel: "Next",
      internTitle: "Internship — your company?", internBody: "Looking for a final internship (eindstage) in software development, from around August 2027.",
      moreEyebrow: "Beyond school", moreTitle: "Hobbies, certificates & more",
      cert: "Certificate", certName: "First Certificate in English · C1", issuer: "Cambridge English · 2025",
      badminton: "Competitive badminton", club: "Playing competition since 2024",
      chess: "Chess", chessLine: "I love to play chess",
      other: "Other", aiTitle: "Building with AI", aiLine: "I like making things together with AI tools. This site is one of them.",
      contactTitle: "Looking for an intern? Let’s talk."
    },
    nl: {
      navProjects: "Projecten", navNN: "Neuraal netwerk", navWork: "Werk & school", navMore: "Meer",
      langLabel: "Taal", darkLabel: "Donkere modus",
      badge: "Beschikbaar voor eindstage · vanaf augustus 2027",
      intro: "Tweedejaars student Software Development aan het Nova College. Ik hou van de onderdelen die je niet ziet: backends, databases en hoe systemen in elkaar grijpen.",
      cta1: "Bekijk mijn projecten", cta2: "Probeer het neurale netwerk",
      langsTitle: "Talen waarin ik programmeer", focus: "Ik probeer nu van alles uit; backend is waar ik naartoe wil.",
      projTitle: "Wat ik heb gebouwd", project: "project", projects: "projecten",
      prev: "Vorige projecten", next: "Volgende projecten", viewGh: "Bekijk op GitHub →",
      noDesc: "Nog geen beschrijving.", allTitle: "Meer op GitHub",
      allDesc: "Al mijn openbare repositories staan op mijn GitHub-profiel.", allLink: "Open mijn GitHub-profiel →",
      nnTitle: "Teken een cijfer. Kijk hoe het denkt.",
      nnBody: "Een neuraal netwerk, vanaf nul gebouwd, dat in je browser draait. Teken een cijfer en zie elke laag oplichten voordat het een gok doet.",
      noLibs: "Geen ML-libraries", openDemo: "Open de volledige demo",
      demoLabel: "Live demo", clear: "Wissen", pad: "Tekenvlak", input: "Teken hier een cijfer · 0–9", guess: "Gok",
      demoError: "De demo kon hier niet laden. Open in plaats daarvan de volledige demo.",
      wsTitle: "Waar ik nu sta",
      wsBody: "Ik richt me op backend-development, databases en systeemarchitectuur, met op de lange termijn interesse in AI en machine learning.",
      school: "Opleiding", work: "Werk", now: "Nu",
      nova: "Nova College · tweede jaar", prevSchool: "Dali College", level: "Mavo",
      vomarRole: "Bijbaan · Vakkenvuller / Kassamedewerker", motiRole: "Blokstage · werk aan een planningstool", nextLabel: "Volgende stap",
      internTitle: "Stage — bij jouw bedrijf?", internBody: "Op zoek naar een eindstage in software development, vanaf ongeveer augustus 2027.",
      moreEyebrow: "Naast school", moreTitle: "Hobby’s, certificaten & meer",
      cert: "Certificaat", certName: "First Certificate in English · C1", issuer: "Cambridge English · 2025",
      badminton: "Competitiebadminton", club: "Speel competitie sinds 2024",
      chess: "Schaken", chessLine: "Ik schaak graag",
      other: "Overig", aiTitle: "Bouwen met AI", aiLine: "Ik maak graag dingen samen met AI-tools. Deze site is er één van.",
      contactTitle: "Op zoek naar een stagiair? Laten we praten."
    }
  };

  var root = document.documentElement;
  var lang = root.lang === "nl" ? "nl" : "en";
  var repos = null; // null = still loading, [] = none / failed

  function remember(key, value) {
    try { localStorage.setItem(key, value); } catch (e) { /* private mode: just don't persist */ }
  }

  // ------------------------------------------------------------- language

  function applyLang() {
    var t = STRINGS[lang];
    root.lang = lang;
    document.querySelectorAll("[data-i18n]").forEach(function (el) {
      var text = t[el.getAttribute("data-i18n")];
      if (text !== undefined) el.textContent = text;
    });
    document.querySelectorAll("[data-i18n-aria-label]").forEach(function (el) {
      var text = t[el.getAttribute("data-i18n-aria-label")];
      if (text !== undefined) el.setAttribute("aria-label", text);
    });
    document.querySelectorAll("[data-lang]").forEach(function (button) {
      button.setAttribute("aria-pressed", String(button.getAttribute("data-lang") === lang));
    });
    renderProjects();
  }

  document.querySelectorAll("[data-lang]").forEach(function (button) {
    button.addEventListener("click", function () {
      lang = button.getAttribute("data-lang");
      remember("cv-lang", lang);
      applyLang();
    });
  });

  // ---------------------------------------------------------------- theme

  var themeToggle = document.getElementById("theme-toggle");

  function isDark() { return root.classList.contains("t-dark"); }

  function syncTheme() { themeToggle.setAttribute("aria-checked", String(isDark())); }

  themeToggle.addEventListener("click", function () {
    var next = isDark() ? "light" : "dark";
    root.className = "t-" + next;
    remember("cv-theme", next);
    syncTheme();
  });

  // ------------------------------------------------------------- projects

  var track = document.getElementById("projects-track");
  var count = document.getElementById("project-count");

  function el(tag, className, text) {
    var node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
  }

  // `image` is the picture set on GitHub (repository Settings → Social preview);
  // without one, or if it fails to load, the cover shows the name instead.
  function projectCard(cover, kind, title, desc, tags, href, linkText, image) {
    var card = el("article", "card project");
    var coverEl = el("div", "project-cover", cover);
    if (image) {
      var img = document.createElement("img");
      img.alt = "";
      img.loading = "lazy";
      img.addEventListener("error", function () { coverEl.replaceChildren(cover); });
      img.src = image;
      coverEl.replaceChildren(img);
    }
    card.append(coverEl);
    var body = el("div", "project-body");
    if (kind) body.append(el("span", "label", kind));
    body.append(el("h3", "title", title), el("p", "", desc));
    if (tags.length) {
      var list = el("div", "tags");
      tags.forEach(function (tag) { list.append(el("span", "tag", tag)); });
      body.append(list);
    }
    var link = el("a", "project-link", linkText);
    link.href = href;
    link.rel = "noopener";
    body.append(link);
    card.append(body);
    return card;
  }

  function renderProjects() {
    var t = STRINGS[lang];
    track.replaceChildren();
    (repos || []).forEach(function (repo) {
      track.append(projectCard(
        repo.name, repo.language, repo.name, repo.description || t.noDesc,
        (repo.topics || []).slice(0, 4), repo.url, t.viewGh, repo.image
      ));
    });
    if (repos !== null) {
      track.append(projectCard("GitHub", "", t.allTitle, t.allDesc, [], "https://github.com/" + GITHUB_USER, t.allLink));
    }
    var n = (repos || []).length;
    count.textContent = n ? n + " " + (n === 1 ? t.project : t.projects) : "";
  }

  // assets/projects.json holds the repositories pinned on the GitHub profile.
  // It is refreshed on every deploy (see tools/update-projects.sh).
  function loadProjects() {
    fetch("assets/projects.json")
      .then(function (response) {
        if (!response.ok) throw new Error("projects.json: HTTP " + response.status);
        return response.json();
      })
      .then(function (list) { repos = list.slice(0, MAX_PROJECTS); })
      .catch(function (error) {
        console.warn("Could not load the projects.", error);
        repos = [];
      })
      .then(renderProjects);
  }

  function slide(direction) {
    var card = track.querySelector(".project");
    var step = card ? card.getBoundingClientRect().width + 32 : 392;
    var reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    track.scrollBy({ left: direction * step, behavior: reduced ? "auto" : "smooth" });
  }
  document.getElementById("projects-prev").addEventListener("click", function () { slide(-1); });
  document.getElementById("projects-next").addEventListener("click", function () { slide(1); });

  syncTheme();
  applyLang();
  loadProjects();
})();
