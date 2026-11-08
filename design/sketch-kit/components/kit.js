var h = window.React.createElement;
var MARKERS = ["butter", "coral", "mint", "sky", "lavender", "sage", "peach", "powder"];
var TABS = [
  { id: "library", label: "Library", icon: "tab_library" },
  { id: "practice", label: "Practice", icon: "tab_practice" },
  { id: "routines", label: "Routines", icon: "tab_routines" },
  { id: "progress", label: "Progress", icon: "tab_progress" },
];

function markerVar(marker) {
  return "var(--marker-" + (MARKERS.indexOf(marker) >= 0 ? marker : "butter") + ")";
}

function type(style, extra) {
  return extra ? style + " " + extra : style;
}

function Icon(props) {
  var icon = ICONS[props.name];
  if (!icon) return null;
  return h("svg", {
    className: "ik-icon ik-icon--" + (props.size || "inline") + (props.className ? " " + props.className : ""),
    viewBox: icon.viewBox,
    "aria-hidden": "true",
    dangerouslySetInnerHTML: { __html: icon.body },
  });
}

// Card: ios/Intrada/DesignSystem/CardSurface.swift, CardShadow.swift; android ui/components/CardSurface.kt, CardShadow.kt
function Card(props) {
  var cls = "ik-card" + (props.lift === false ? "" : " ik-card--lift") + (props.compact ? " ik-card--compact" : "");
  return h("div", { className: cls, style: props.style }, props.children);
}

// MarkerButton: ios/Intrada/DesignSystem/MarkerButton.swift; android ui/FirstRunScreen.kt
function MarkerButton(props) {
  return h(
    "button",
    { type: "button", className: type("button", "ik-marker-button"), style: { background: markerVar(props.marker) } },
    props.label
  );
}

function ScoreMark(props) {
  var score = typeof props.score === "number" ? Math.max(0, Math.min(10, props.score)) : null;
  var arc = score === null ? 0 : (score / 10) * 87.96;
  return h(
    "span",
    { className: "ik-score", "aria-label": score === null ? "No mark yet" : "Mark " + score },
    h(
      "svg",
      { viewBox: "0 0 32 32", "aria-hidden": "true" },
      h("circle", { className: "ik-score__track", cx: 16, cy: 16, r: 14 }),
      h("circle", { className: "ik-score__fill", cx: 16, cy: 16, r: 14, strokeDasharray: arc + " 88" })
    ),
    score === null ? null : h("span", { className: type("smallMedium", "ik-score__figure") }, String(score))
  );
}

function Chip(props) {
  return h("span", { className: type("secondary", "ik-chip") }, props.lead || null, props.children);
}

// LibraryItemCard: ios/Intrada/Views/Components/LibraryItemCard.swift; android ui/LibraryScreen.kt
function LibraryItemCard(props) {
  var kind = props.kind === "exercise" ? "exercise" : "piece";
  var meta = [props.keyName, props.tempo].filter(Boolean).join(" · ");
  var prompt = !props.subtitle && !meta ? (kind === "piece" ? "Add composer, key and tempo" : "Add key and tempo") : null;
  var linked = kind === "piece" ? props.linkedExercises || 0 : 0;
  var tags = props.tags || [];
  var extras =
    props.priority || linked > 0 || props.keys > 1 || props.variations > 1 || tags.length
      ? h(
          "div",
          { className: "ik-item__extras" },
          props.priority ? h(Icon, { name: "star", size: "caption" }) : null,
          linked > 0 ? h(Chip, null, String(linked)) : null,
          props.keys > 1 ? h(Chip, { lead: "\u266f" }, props.keys + " keys") : null,
          props.variations > 1 ? h(Chip, null, props.variations + " variations") : null,
          tags.map(function (tag) {
            return h(Chip, { key: tag }, tag);
          })
        )
      : null;
  return h(
    "div",
    { className: "ik-card ik-card--lift ik-item ik-item--" + kind },
    h(
      "div",
      { className: "ik-item__text" },
      h("div", { className: type("cardTitle", "ik-ink") }, props.title),
      props.subtitle ? h("div", { className: type("secondary", "ik-ink-secondary") }, props.subtitle) : null,
      meta ? h("div", { className: type("secondary", "ik-ink-secondary") }, meta) : null,
      prompt ? h("div", { className: type("secondary", "ik-ink-secondary") }, prompt) : null,
      extras
    ),
    props.showsMastery === false ? null : h(ScoreMark, { score: props.score })
  );
}

// TabBar: ios/Intrada/Views/RootView.swift; android ui/AppFrame.kt, ui/AppTab.kt
function TabBar(props) {
  var device = props.device === "android" ? "android" : "ios";
  var selected = props.selected || "library";
  return h(
    "nav",
    { className: "ik-tabbar ik-tabbar--" + device, role: "tablist" },
    h(
      "div",
      { className: "ik-tabbar__row" },
      TABS.map(function (tab) {
        var on = tab.id === selected;
        return h(
          "div",
          { key: tab.id, role: "tab", "aria-selected": on, className: "ik-tab" + (on ? " ik-tab--on" : "") },
          h(Icon, { name: tab.icon, size: device === "ios" ? "tab" : "android-tab" }),
          h("span", { className: type(device === "ios" ? "smallMedium" : "small") }, tab.label)
        );
      })
    ),
    h("div", { className: "ik-tabbar__inset" })
  );
}

function StatusBar(props) {
  return h(
    "div",
    { className: "ik-status ik-status--" + props.device, "aria-hidden": "true" },
    h("span", { className: type("smallMedium") }, "9:41"),
    h("span", { className: "ik-status__battery" })
  );
}

// ScreenScaffold: ios/Intrada/DesignSystem/ScreenScaffold.swift; android ui/ScreenScaffold.kt
function ScreenScaffold(props) {
  var device = props.device === "android" ? "android" : props.device === "ios" ? "ios" : null;
  var add = props.addLabel
    ? h(
        "button",
        { type: "button", className: "ik-add", "aria-label": props.addLabel },
        h("span", { className: "ik-add__disc" }, h(Icon, { name: "plus" }))
      )
    : null;
  var swipe = device !== "android" && !props.noSwipe;
  var screen = h(
    "div",
    { className: "ik-scaffold" },
    device === "ios" && props.pushed
      ? h("div", { className: "ik-navbar" }, h(Icon, { name: "chevron_right", size: "control", className: "ik-back" }))
      : null,
    h(
      "header",
      { className: "ik-scaffold__header" },
      h(
        "div",
        { className: "ik-scaffold__titles" },
        h(
          "h1",
          { className: type("pageTitle", "ik-ink ik-scaffold__title") },
          h("span", { className: swipe ? "ik-swipe" : null, style: { "--ik-marker": markerVar(props.marker) } }, props.title)
        ),
        props.subtitle
          ? h("div", { className: type("secondary", "ik-ink-secondary") }, props.subtitle)
          : props.reserveSubtitle
            ? h("div", { className: type("secondary", "ik-hidden"), "aria-hidden": "true" }, "\u00a0")
            : null
      ),
      props.actions || null,
      add
    ),
    h("div", { className: "ik-scaffold__rule" }),
    h("div", { className: "ik-scaffold__content" }, props.children)
  );
  if (!device) return screen;
  return h(
    "div",
    { className: "ik-device ik-device--" + device },
    h(StatusBar, { device: device }),
    screen,
    props.tab ? h(TabBar, { device: device, selected: props.tab }) : h("div", { className: "ik-device__inset" })
  );
}

window.Intrada = {
  ScreenScaffold: ScreenScaffold,
  Card: Card,
  LibraryItemCard: LibraryItemCard,
  MarkerButton: MarkerButton,
  TabBar: TabBar,
};
