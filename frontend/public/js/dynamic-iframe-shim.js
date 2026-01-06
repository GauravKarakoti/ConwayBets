(() => {
  const matchesAuthHost = (url) => {
    try {
      const u = new URL(url, document.baseURI);
      return u.hostname === "relay.dynamicauth.com" || /\.dynamicauth\.com$/.test(u.hostname);
    } catch {
      return false;
    }
  };

  const createElement = document.createElement.bind(document);
  document.createElement = (tagName, options) => {
    const el = createElement(tagName, options);
    if (tagName.toLowerCase() === "iframe") {
      const setAttribute = el.setAttribute.bind(el);
      Object.defineProperty(el, "src", {
        set: (url) => {
          if (matchesAuthHost(url)) {
            setAttribute("credentialless", "");
          }
          setAttribute("src", url);
        },
        get: () => el.getAttribute("src"),
      });
    }
    return el;
  };
})();