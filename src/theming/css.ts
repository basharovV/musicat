import { get } from "svelte/store";
import processConfig from "./processConfig";
import {
    currentFont,
    currentThemeObject
} from "./store";

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
    themeVariableName
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
 * Create CSS template
 * @name createCSSTemplate
 * @param {string} prefix - CSS variable prefix
 * @param {Object[]} themes - themes array
 * @returns {string} CSS template
 */
export function createCSSTemplate(prefix, base = {}) {
    const variablePrefix = prefix ? `--${prefix}` : "-";

    const theme = get(currentThemeObject);
    const themeCSS = Object.entries(theme).reduce((acc, val) => {
        if (val[0] === "type" && val[1] === "dark") {
            acc += `--inverse: #ffffff2f;\n`;
            acc += `--type-bw: black;\n`;
            acc += `--type-bw-inverse: white;\n`;
        } else if (val[0] === "type" && val[1] === "light") {
            acc += `--inverse: black;\n`;
            acc += `--type-bw: white;\n`;
            acc += `--type-bw-inverse: black;\n`;
        }
        return (acc += `--${val[0]}: ${val[1]};\n`);
    }, "");
    const font = get(currentFont);

    const template = `
    <style>
      :root {
      ${themeCSS}
      font-family: ${font};
      }
    </style>
  `;

    return template.trim();
}
