// English and Dutch text for the full demo. The choice is shared with the CV
// page (same localStorage key), so switching on one page carries over.

const STRINGS = {
  en: {
    pageTitle: "Watch a neural network read your handwriting",
    back: "Back to CV",
    skip: "Skip to content",
    langLabel: "Language",
    darkLabel: "Dark mode",
    title: "Watch a neural network read your handwriting",
    lead: "Draw a number. A small neural network, written from scratch in Rust and running right here in your browser, lights up layer by layer as it decides what you drew.",
    noLibs: "No ML libraries",
    switchLabel: "Which weights the network uses",
    weightsClaude: "Claude-picked weights",
    weightsTrained: "Trained weights",
    claude1:
      "<strong>An experiment to see how good AI is at making AI.</strong> Nobody trained this version. Claude, an AI model, chose every one of the 25,818 weights itself by reasoning about what handwritten numbers look like, with no training data and without ever correcting a mistake. It has never seen a real handwritten number.",
    claude2: "On 10,000 real handwritten numbers it gets 39.7% right. Pure chance would be 10%; the trained version gets 96.8%.",
    trained1:
      "<strong>Learned from 70,000 handwritten numbers.</strong> This version started with completely random weights, so at first it was right only by luck. It was then shown every image in the MNIST collection of handwritten numbers, 70,000 of them, 20 times over. For each image it made its guess, the guess was compared with the right answer, and every weight was nudged a tiny bit in the direction that would have made the answer more right (backpropagation). That happened in small groups of 32 images: 43,760 rounds of nudging all 25,818 weights, about 25 seconds on a laptop.",
    trained2:
      "Trained the same way on 60,000 of the images, it reads 96.8% of the other 10,000 correctly, images it had never seen. The version running here then used those 10,000 as well.",
    step1: "1. Draw a number (0–9)",
    padLabel: "Drawing area",
    clear: "Clear",
    drawHint: "Mouse, finger or pen. Draw big.",
    step2: "2. What the network sees",
    seenHint: "Your drawing, cropped, shrunk to 28×28 pixels and centred.",
    step3: "3. Its answer",
    drawToStart: "Draw something to start.",
    notSure: "Not sure: {pct}.",
    sure: "{pct} sure.",
    couldBe: " Could also be a {digit} ({pct}).",
    inside: "Inside the network",
    loading: "Loading…",
    status: "{sizes} neurons, Rust compiled to WebAssembly",
    loadFailed: "Could not load the network.",
    loadFailedFile: "WebAssembly can't load from file://. Serve this folder over HTTP (see the README).",
    loadFailedWhy: "Could not load the network: {message}",
    legend:
      '<span class="swatch warm"></span> pushes a neuron on <span class="swatch cool"></span> holds it back. Brighter means more active. Hover over or tap a neuron to see what it looks for.',
    howTitle: "What am I looking at?",
    how1: "On the left is your drawing as 784 pixels. Each circle after that is a neuron: it adds up what the previous layer tells it, and lights up if the total is high enough. The last layer has ten neurons, one per number; the brightest one is the answer.",
    how2: "Both versions are the same network with the same 58 neurons; only the weights differ. Draw something and flip the switch at the top to see how differently they react.",
    // Drawn on the network canvas and in its tooltip.
    netInput: "input, 784 pixels",
    netInputShort: "input",
    netAnswer: "answer",
    netHidden: "hidden layer {k}",
    netHiddenShort: "hidden",
    neuron: "neuron {n}",
    theDigit: "the number {d}",
    tipSure: "{pct}% sure",
    tipActivation: "activation {value}",
    tipIdle: "idle",
    tipWantsInk: "wants ink here",
    tipWantsEmpty: "wants it empty",
    tipStrongest: "Strongest influences right now",
    tipListens: "Listens most to",
  },
  nl: {
    pageTitle: "Kijk hoe een neuraal netwerk je handschrift leest",
    back: "Terug naar het cv",
    skip: "Naar de inhoud",
    langLabel: "Taal",
    darkLabel: "Donkere modus",
    title: "Kijk hoe een neuraal netwerk je handschrift leest",
    lead: "Teken een cijfer. Een klein neuraal netwerk, vanaf nul geschreven in Rust en draaiend in je browser, licht laag voor laag op terwijl het bepaalt wat je hebt getekend.",
    noLibs: "Geen ML-libraries",
    switchLabel: "Welke gewichten het netwerk gebruikt",
    weightsClaude: "Door Claude gekozen gewichten",
    weightsTrained: "Getrainde gewichten",
    claude1:
      "<strong>Een experiment om te zien hoe goed AI is in het maken van AI.</strong> Niemand heeft deze versie getraind. Claude, een AI-model, koos elk van de 25.818 gewichten zelf door te beredeneren hoe handgeschreven cijfers eruitzien, zonder trainingsdata en zonder ooit een fout te corrigeren. Het heeft nog nooit een echt handgeschreven cijfer gezien.",
    claude2: "Van 10.000 echte handgeschreven cijfers heeft het 39,7% goed. Puur gokken zou 10% opleveren; de getrainde versie haalt 96,8%.",
    trained1:
      "<strong>Geleerd van 70.000 handgeschreven cijfers.</strong> Deze versie begon met volledig willekeurige gewichten, dus in het begin had het alleen bij toeval gelijk. Daarna kreeg het elke afbeelding uit de MNIST-verzameling van handgeschreven cijfers te zien, 70.000 stuks, 20 keer achter elkaar. Bij elke afbeelding deed het een gok, die gok werd vergeleken met het juiste antwoord, en elk gewicht werd een heel klein beetje bijgestuurd in de richting die het antwoord beter zou hebben gemaakt (backpropagation). Dat gebeurde in kleine groepjes van 32 afbeeldingen: 43.760 rondes waarin alle 25.818 gewichten werden bijgestuurd, ongeveer 25 seconden op een laptop.",
    trained2:
      "Op dezelfde manier getraind op 60.000 van de afbeeldingen leest het 96,8% van de overige 10.000 goed, afbeeldingen die het nog nooit had gezien. De versie die hier draait heeft daarna ook die 10.000 gebruikt.",
    step1: "1. Teken een cijfer (0–9)",
    padLabel: "Tekenvlak",
    clear: "Wissen",
    drawHint: "Muis, vinger of pen. Teken groot.",
    step2: "2. Wat het netwerk ziet",
    seenHint: "Je tekening, bijgesneden, verkleind naar 28×28 pixels en gecentreerd.",
    step3: "3. Het antwoord",
    drawToStart: "Teken iets om te beginnen.",
    notSure: "Niet zeker: {pct}.",
    sure: "{pct} zeker.",
    couldBe: " Zou ook een {digit} kunnen zijn ({pct}).",
    inside: "In het netwerk",
    loading: "Laden…",
    status: "{sizes} neuronen, Rust gecompileerd naar WebAssembly",
    loadFailed: "Het netwerk kon niet worden geladen.",
    loadFailedFile: "WebAssembly kan niet laden vanaf file://. Serveer deze map via HTTP (zie de README).",
    loadFailedWhy: "Het netwerk kon niet worden geladen: {message}",
    legend:
      '<span class="swatch warm"></span> duwt een neuron aan <span class="swatch cool"></span> houdt het tegen. Feller betekent actiever. Beweeg over een neuron of tik erop om te zien waar het op let.',
    howTitle: "Waar kijk ik naar?",
    how1: "Links staat je tekening als 784 pixels. Elk rondje daarna is een neuron: het telt op wat de vorige laag doorgeeft en licht op als het totaal hoog genoeg is. De laatste laag heeft tien neuronen, één per cijfer; de felste is het antwoord.",
    how2: "Beide versies zijn hetzelfde netwerk met dezelfde 58 neuronen; alleen de gewichten verschillen. Teken iets en zet de schakelaar bovenaan om, dan zie je hoe verschillend ze reageren.",
    netInput: "invoer, 784 pixels",
    netInputShort: "invoer",
    netAnswer: "antwoord",
    netHidden: "verborgen laag {k}",
    netHiddenShort: "laag {k}",
    neuron: "neuron {n}",
    theDigit: "het cijfer {d}",
    tipSure: "{pct}% zeker",
    tipActivation: "activatie {value}",
    tipIdle: "inactief",
    tipWantsInk: "wil hier inkt",
    tipWantsEmpty: "wil het leeg",
    tipStrongest: "Sterkste invloeden op dit moment",
    tipListens: "Luistert het meest naar",
  },
};

let lang = document.documentElement.lang === "nl" ? "nl" : "en";
const listeners = [];

/** The text for `key` in the current language; `{name}` parts are filled from `vars`. */
export function t(key, vars = {}) {
  const text = STRINGS[lang][key] ?? STRINGS.en[key] ?? key;
  return text.replace(/\{(\w+)\}/g, (_, name) => String(vars[name] ?? ""));
}

/** Numbers follow the language too (39.7 / 39,7). */
export function formatNumber(value, digits) {
  return value.toLocaleString(lang === "nl" ? "nl-NL" : "en-US", {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  });
}

/** Called after every language change, once the static text has been replaced. */
export function onLangChange(listener) {
  listeners.push(listener);
}

export function applyLang() {
  document.documentElement.lang = lang;
  document.title = t("pageTitle");
  for (const el of document.querySelectorAll("[data-i18n]")) el.textContent = t(el.dataset.i18n);
  // Only our own strings above, which may contain markup.
  for (const el of document.querySelectorAll("[data-i18n-html]")) el.innerHTML = t(el.dataset.i18nHtml);
  for (const el of document.querySelectorAll("[data-i18n-aria-label]")) el.setAttribute("aria-label", t(el.dataset.i18nAriaLabel));
  for (const button of document.querySelectorAll("[data-lang]")) button.setAttribute("aria-pressed", String(button.dataset.lang === lang));
}

for (const button of document.querySelectorAll("[data-lang]")) {
  button.addEventListener("click", () => {
    lang = button.dataset.lang === "nl" ? "nl" : "en";
    try {
      localStorage.setItem("cv-lang", lang);
    } catch {
      // Private mode: the choice just isn't remembered.
    }
    applyLang();
    for (const listener of listeners) listener();
  });
}

applyLang();
