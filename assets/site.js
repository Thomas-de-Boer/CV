// Language switch, theme switch and the projects slider. No framework.
(function () {
  "use strict";

  // The last project card always links to this GitHub profile.
  var GITHUB_USER = "Thomas-de-Boer";
  var MAX_PROJECTS = 6;

  var STRINGS = {
    en: {
      skip: "Skip to content",
      navProjects: "Projects", navNN: "Neural network", navWork: "Work & school", navMore: "Other",
      langLabel: "Language", darkLabel: "Dark mode",
      intro: "I’m a second-year Software Development student (MBO 4) at Nova College. I’m most interested in backend work: databases and how systems fit together. I’m looking for a final internship from August 2027.",
      langs: "I’ve worked in Java, Python, C#, Rust and JavaScript, and I’m still trying a bit of everything.",
      cta1: "Projects", cta2: "Neural network demo",
      project: "project", projects: "projects",
      prev: "Previous projects", next: "Next projects", viewGh: "View on GitHub",
      noDesc: "No description yet.", allTitle: "More on GitHub",
      allDesc: "All my public repositories are on my GitHub profile.", allLink: "Open my GitHub profile",
      nnTitle: "A neural network I wrote from scratch",
      nnBody: "It’s written in Rust without any machine learning libraries and runs in your browser as WebAssembly. Draw a digit on the pad and you can see what each layer does before it guesses.",
      openDemo: "Open the full demo",
      demoLabel: "Try it", clear: "Clear", pad: "Drawing pad", input: "Draw a digit here (0–9)", guess: "Guess",
      demoError: "The demo could not load here. Open the full demo instead.",
      wsBody: "I want to grow into backend development, databases and systems architecture, and later on AI and machine learning.",
      school: "School", work: "Work", now: "now",
      nova: "Nova College, second year", prevSchool: "Dali College", level: "Mavo",
      vomarRole: "Part-time shelf stocker and cashier", motiRole: "Internship (blokstage), working on a planning tool",
      internTitle: "Final internship", internBody: "From around August 2027. I’m still looking for a company.",
      cert: "Certificate", certName: "Cambridge First Certificate in English (C1)",
      badminton: "Badminton", club: "Playing in competition since 2024",
      chess: "Chess", chessLine: "I love to play chess",
      other: "Other", aiTitle: "Building with AI", aiLine: "I like making things together with AI tools. This site is one of them.",
      contactBody: "Email is the easiest way to reach me."
    },
    nl: {
      skip: "Naar de inhoud",
      navProjects: "Projecten", navNN: "Neuraal netwerk", navWork: "Werk & school", navMore: "Overig",
      langLabel: "Taal", darkLabel: "Donkere modus",
      intro: "Ik ben tweedejaars student Software Development (MBO 4) aan het Nova College. Ik ben het meest geïnteresseerd in backend: databases en hoe systemen in elkaar grijpen. Vanaf augustus 2027 zoek ik een eindstage.",
      langs: "Ik heb gewerkt met Java, Python, C#, Rust en JavaScript, en ik probeer nog van alles uit.",
      cta1: "Projecten", cta2: "Demo neuraal netwerk",
      project: "project", projects: "projecten",
      prev: "Vorige projecten", next: "Volgende projecten", viewGh: "Bekijk op GitHub",
      noDesc: "Nog geen beschrijving.", allTitle: "Meer op GitHub",
      allDesc: "Al mijn openbare repositories staan op mijn GitHub-profiel.", allLink: "Open mijn GitHub-profiel",
      nnTitle: "Een neuraal netwerk dat ik zelf heb geschreven",
      nnBody: "Het is geschreven in Rust zonder machine-learning-libraries en draait als WebAssembly in je browser. Teken een cijfer en je ziet wat elke laag doet voordat het een gok doet.",
      openDemo: "Open de volledige demo",
      demoLabel: "Probeer het", clear: "Wissen", pad: "Tekenvlak", input: "Teken hier een cijfer (0–9)", guess: "Gok",
      demoError: "De demo kon hier niet laden. Open in plaats daarvan de volledige demo.",
      wsBody: "Ik wil verder in backend-development, databases en systeemarchitectuur, en later ook AI en machine learning.",
      school: "Opleiding", work: "Werk", now: "nu",
      nova: "Nova College, tweede jaar", prevSchool: "Dali College", level: "Mavo",
      vomarRole: "Bijbaan als vakkenvuller en kassamedewerker", motiRole: "Blokstage, werk aan een planningstool",
      internTitle: "Eindstage", internBody: "Vanaf ongeveer augustus 2027. Ik zoek nog een bedrijf.",
      cert: "Certificaat", certName: "Cambridge First Certificate in English (C1)",
      badminton: "Badminton", club: "Speel competitie sinds 2024",
      chess: "Schaken", chessLine: "Ik schaak graag",
      other: "Overig", aiTitle: "Bouwen met AI", aiLine: "Ik maak graag dingen samen met AI-tools. Deze site is er één van.",
      contactBody: "Mailen is de makkelijkste manier om me te bereiken."
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

  function syncTheme() {
    themeToggle.setAttribute("aria-checked", String(isDark()));
    var meta = document.querySelector('meta[name="theme-color"]');
    if (meta) meta.content = isDark() ? "#151515" : "#faf9f6";
  }

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
  // without one, or if it fails to load, the card has no picture at all.
  function projectCard(kind, title, desc, tags, href, linkText, image) {
    var card = el("article", "card project");
    if (image) {
      var coverEl = el("div", "project-cover");
      var img = document.createElement("img");
      img.alt = "";
      img.loading = "lazy";
      img.addEventListener("error", function () { coverEl.remove(); });
      img.src = image;
      coverEl.append(img);
      card.append(coverEl);
    }
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
    // Until the list has loaded, keep the cards written into the page at build time.
    if (repos === null) return;
    track.replaceChildren();
    (repos || []).forEach(function (repo) {
      track.append(projectCard(
        repo.language, repo.name, repo.description || t.noDesc,
        (repo.topics || []).slice(0, 4), repo.url, t.viewGh, repo.image
      ));
    });
    track.append(projectCard("", t.allTitle, t.allDesc, [], "https://github.com/" + GITHUB_USER, t.allLink));
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
