const {fontFamily} = require('tailwindcss/defaultTheme');

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./src/templates/**/*.html'],
  theme: {
    colors: {
      // Colors from Prodeko's style guide, https://static.prodeko.org/media/public/2023/12/14/brandidokumentti_final.pdf 
      'on-primary': '#f6f6f6', // Prodeko's official "valkoinen 1"
      'on-secondary': '#f6f6f6', // Prodeko's official "valkoinen 1"
      'primary-container': '#ececec', // Prodeko's official "valkoinen 2"
      'on-primary-container': '#000A14', // Prodeko's official "musta"
      'primary': '#002e7d', // Prodeko's official "keskisininen"
      'secondary': '#002851', // Prodekos official "tummansininen"
    },
    fontFamily: {
      sans: ['Raleway', ...fontFamily.sans]
    }  
  },
};
