for (const host of document.querySelectorAll(".example-rendered")) {
  const root = host.shadowRoot;

  for (const el of root.querySelectorAll("[data-math-style]")) {
    katex.render(el.textContent, el, {
      displayMode: el.dataset.mathStyle === "display",
      throwOnError: false,
    });
  }

  root.addEventListener("click", (event) => {
    const link = event.target.closest("a[href]");
    if (!link) return;

    event.preventDefault();

    const href = link.getAttribute("href");
    if (href.startsWith("#")) {
      root
        .getElementById(decodeURIComponent(href.slice(1)))
        .scrollIntoView({ block: "center" });
    }
  });
}

const VIEW_KEY = "comrak-example-view";

function selectView(view) {
  for (const el of document.querySelectorAll(".example-view, .example-flag")) {
    el.hidden = el.dataset.view !== view;
  }
  for (const button of document.querySelectorAll(".example-tabs button")) {
    button.setAttribute("aria-pressed", button.dataset.view === view);
  }
  try {
    localStorage.setItem(VIEW_KEY, view);
  } catch {}
}

for (const tabs of document.querySelectorAll(".example-tabs")) {
  tabs.hidden = false;

  tabs.addEventListener("click", (event) => {
    const button = event.target.closest("button");
    if (!button) return;

    const before = tabs.getBoundingClientRect().top;
    selectView(button.dataset.view);
    window.scrollBy(0, tabs.getBoundingClientRect().top - before);
  });
}

let saved = null;
try {
  saved = localStorage.getItem(VIEW_KEY);
} catch {}
selectView(
  document.querySelector(`.example-tabs button[data-view="${saved}"]`)
    ? saved
    : "rendered",
);

const sidebar = document.querySelector(".docs-nav");
const current = sidebar.querySelector('[aria-current="page"]');
sidebar.scrollTop =
  current.offsetTop - (sidebar.clientHeight - current.offsetHeight) / 2;
