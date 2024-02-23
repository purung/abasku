serve:
	trunk serve --port 3000

css:
	direnv exec . tailwindcss -w -i input.css -o style/output.css
