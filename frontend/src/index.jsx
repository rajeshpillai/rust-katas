/* @refresh reload */
import { render } from "solid-js/web";
import { createSignal, onCleanup, Switch, Match } from "solid-js";
import { initTheme } from "./state";
import Landing from "./pages/landing";
import KataBrowser from "./pages/kata-browser";
import "./index.css";

function App() {
  const [route, setRoute] = createSignal(window.location.hash || "#/");

  function onHashChange() {
    setRoute(window.location.hash || "#/");
  }

  window.addEventListener("hashchange", onHashChange);
  onCleanup(() => window.removeEventListener("hashchange", onHashChange));

  initTheme();

  return (
    <Switch fallback={<Landing />}>
      <Match when={route() === "#/" || route() === ""}>
        <Landing />
      </Match>
      <Match when={route().startsWith("#/katas")}>
        <KataBrowser />
      </Match>
    </Switch>
  );
}

render(() => <App />, document.getElementById("app"));
