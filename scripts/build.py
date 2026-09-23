import os

import rcssmin
import rjsmin

BASE_DIR = os.path.abspath(os.path.dirname(__file__))


def minify_assets():
    print("[BUILD] Iniciando minificação de assets...")

    # css
    css_path = os.path.join(BASE_DIR, "style.css")
    css_min_path = os.path.join(BASE_DIR, "style.min.css")
    if os.path.exists(css_path):
        with open(css_path, "r", encoding="utf-8") as f:
            minified_css = rcssmin.cssmin(f.read())
        with open(css_min_path, "w", encoding="utf-8") as f:
            f.write(minified_css)
        print(f"  -> style.css minificado ({len(minified_css)} bytes).")

    # js
    js_path = os.path.join(BASE_DIR, "script.js")
    js_min_path = os.path.join(BASE_DIR, "script.min.js")
    if os.path.exists(js_path):
        with open(js_path, "r", encoding="utf-8") as f:
            minified_js = rjsmin.jsmin(f.read())
        with open(js_min_path, "w", encoding="utf-8") as f:
            f.write(minified_js)
        print(f"  -> script.js minificado ({len(minified_js)} bytes).")

    print("[BUILD] Concluído.")


if __name__ == "__main__":
    minify_assets()
