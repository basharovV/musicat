import "./app.css";
import App from "./App.svelte";
import { setup } from "./utils/Logger";

const app = new App({
    target: document.getElementById("app"),
    intro: true,
});

setup();

export default app;
