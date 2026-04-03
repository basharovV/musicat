import { get } from "svelte/store";
import processConfig from "./processConfig";
import { currentFont, currentThemeObject } from "./store";
import { resolveDerived } from "./ColorUtils";

/**
 * CSS Variable Name
 * @typedef {string} CSSVariableName
 */

/**
 * CSS Variable Statement
 * @typedef {string} CSSVariableStatement
 */

/**
 * @typedef {object} CreateCSSVariableNameInput
 * @property {string} variablePrefix
 * @property {string} prop property name
 * @property {string} [key] optional prop prefix
 *
 * Helper to create CSS Variable name string
 * @param {CreateCSSVariableNameInput}
 * @returns {CSSVariableName}
 */
export function createCSSVariableName({ variablePrefix, prop, key }) {
    if (key) return `${variablePrefix}-${key}-${prop}`;
    else return `${variablePrefix}-${prop}`;
}

/**
 * Helper to merge variable name and value to create statement
 * @param {CSSVariableName} variableName CSS Variable name
 * @param {string} value CSS Variable Value
 * @returns {CSSVariableStatement}
 */
export function createCSSVariableStatement(variableName, value) {
    return `${variableName}: ${value};`;
}

/**
 * @typedef {object} CreateCSSVariableOverrideInput
 * @property {CSSVariableName} initialVariableName
 * @property {CSSVariableName} themeVariableName
 *
 * Helper to create variable overrides for themed use
 * @param {CreateCSSVariableOverrideInput}
 * @returns {string}
 */
export function createCSSVariableOverride({
    initialVariableName,
    themeVariableName,
}) {
    return `${initialVariableName}: var(${themeVariableName});`;
}

/**
 *
 * @param {object} config
 * @param {Object} options
 * @param {string} options.prefix
 * @returns {[CSSVariableName, <string,CSSVariableName>]}
 */
export function createCSSVariableCollection(config, { prefix } = {}) {
    const variablePrefix = prefix ? `--${prefix}` : "-";
    const processedConfig = processConfig(config);
    const variables = Object.entries(processedConfig).map(([prop, value]) => {
        return [createCSSVariableName({ variablePrefix, prop }), value];
    });
    return variables;
}

/**
 * Creates a `<style>` block with all theme variables resolved to static hex values.
 * Derived values using `rgb(from var(--x) …)` or `color-mix(…)` are resolved
 * against the core hex values in the theme, ensuring compatibility with contexts
 * that require static colors such as Konva canvas fills.
 *
 * @param prefix - Optional CSS custom property prefix e.g. `"my-app"` → `--my-app-accent`.
 * @param base - Optional base overrides merged into the theme before resolution.
 * @returns A trimmed `<style>` string with resolved CSS custom properties on `:root`.
 */
export function createCSSTemplate(prefix, base = {}) {
    const variablePrefix = prefix ? `--${prefix}` : "-";

    const theme = get(currentThemeObject);
    let themeCSS = Object.entries(theme).reduce((acc, [key, value]) => {
        return (acc += `--${key}: ${value};\n`);
    }, "");

    for (const [key, value] of Object.entries(get(currentFont))) {
        themeCSS += `${key}: ${value};\n`;
    }

    return `
    <style>
      :root {
        ${themeCSS}
      }
    </style>
  `.trim();
}
