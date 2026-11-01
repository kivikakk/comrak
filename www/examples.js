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
