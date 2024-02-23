/** @type {import('tailwindcss').Config} */
module.exports = {
  content: { 
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {
      gridTemplateColumns: {
        'smol': 'repeat(auto-fit, minmax(min(100%, var(--min)), 1fr))'
      }
    },
  },
  daisyui: {
    darkTheme: "night",
    themes: [
      "nord",
      "night"
    ],
  },
  plugins: [require("daisyui"), require("@tailwindcss/forms")],
}
