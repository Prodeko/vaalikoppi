const {fontFamily} = require('tailwindcss/defaultTheme');

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./src/templates/**/*.html'],
  theme: {
    colors: {
      "surface-strong": "var(--surface-strong)",
      "surface-weak": "var(--surface-weak)",
      "surface-primary": "var(--surface-primary)",
      "content-on-strong-strong": "var(--content-on-strong-strong)",
      "content-on-strong-weak": "var(--content-on-strong-weak)",
      "content-on-weak-weak": "var(--content-on-weak-weak)",
      "content-on-weak-strong": "var(--content-on-weak-strong)",
      "content-on-primary": "var(--content-on-primary)",
      "content-primary": "var(--content-primary)",
    },
    fontFamily: {
      sans: ['Raleway', ...fontFamily.sans]
    }  
  },
};
