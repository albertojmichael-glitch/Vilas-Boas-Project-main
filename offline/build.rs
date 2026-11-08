use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../server");
    println!("cargo:rerun-if-changed=../client");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("server_bundle.json");

    let py_script = r#"
import ast
import json
import os
import glob

bundle = {
    "mapa": {},
    "descricoes_itens": {},
    "eventos_aleatorios": [],
    "sinonimos": {},
    "ascii_arts": {},
}

json_candidates = (
    glob.glob("../data/**/*.json", recursive=True) +
    glob.glob("../server/**/*.json", recursive=True) +
    glob.glob("data/**/*.json", recursive=True)
)

for jpath in json_candidates:
    try:
        with open(jpath, "r", encoding="utf-8") as f:
            data = json.load(f)
        if isinstance(data, dict):
            for map_key in ("mapa_original", "mapa"):
                if map_key in data and isinstance(data[map_key], dict):
                    bundle["mapa"].update(data[map_key])
            for k in ("descricoes_itens", "itens_descricoes", "itens_info"):
                if k in data and isinstance(data[k], dict):
                    bundle["descricoes_itens"].update(data[k])
            if "eventos_aleatorios" in data and isinstance(data["eventos_aleatorios"], list):
                bundle["eventos_aleatorios"] = data["eventos_aleatorios"]
            if "sinonimos" in data and isinstance(data["sinonimos"], dict):
                bundle["sinonimos"].update(data["sinonimos"])
    except Exception:
        pass

for pyf in sorted(glob.glob("../server/**/*.py", recursive=True)):
    try:
        with open(pyf, "r", encoding="utf-8") as f:
            tree = ast.parse(f.read())
        for node in ast.walk(tree):
            if isinstance(node, ast.Assign):
                for target in node.targets:
                    if isinstance(target, ast.Name):
                        name = target.id
                        try:
                            val = ast.literal_eval(node.value)
                        except Exception:
                            continue
                        if isinstance(val, str) and (
                            "ARTE" in name.upper()
                            or "CAVEIRA" in name.upper()
                            or "ASCII" in name.upper()
                            or val.count("\n") >= 3
                        ):
                            bundle["ascii_arts"][name] = val
                        elif isinstance(val, dict) and "DESCRIC" in name.upper():
                            for k, v in val.items():
                                if isinstance(v, str):
                                    bundle["descricoes_itens"][str(k)] = v
    except Exception:
        pass

print(json.dumps(bundle, ensure_ascii=False))
"#;

    let output = Command::new("python3")
        .arg("-c")
        .arg(py_script)
        .output()
        .or_else(|_| Command::new("python").arg("-c").arg(py_script).output());

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.trim().starts_with('{') {
                fs::write(&dest_path, stdout.trim()).unwrap();
                return;
            }
        }
    }

    let fallback = fs::read_to_string("../server/data/game_data.json")
        .unwrap_or_else(|_| "{}".to_string());
    fs::write(&dest_path, fallback).unwrap();
}
