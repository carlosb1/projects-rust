import httpx
from typing import Dict, Any, List, Optional

# --- paste your flatten_boe_payload here or import it ---
def flatten_boe_payload(payload: Dict[str, Any]) -> List[Dict[str, Optional[str]]]:
    out = []
    sumario = (payload or {}).get("data", {}).get("sumario", {})
    meta = sumario.get("metadatos", {})
    publicacion = meta.get("publicacion")
    fecha_publicacion = meta.get("fecha_publicacion")

    for diario in sumario.get("diario", []) or []:
        diario_numero = diario.get("numero")
        sd = diario.get("sumario_diario", {}) or {}
        diario_identificador = sd.get("identificador")
        diario_pdf_url = (sd.get("url_pdf") or {}).get("texto")

        for seccion in diario.get("seccion", []) or []:
            print("//////////////")
            print(str(seccion))
            print("//////////////")

            seccion_codigo = seccion.get("codigo")
            seccion_nombre = seccion.get("nombre")

            possib_dept = seccion.get("departamento", [])

            if isinstance(possib_dept, dict):
                dept_codigo = dept.get("codigo", "")
                dept_nombre = dept.get("nombre", "")

                for epi in dept.get("epigrafe", []) or []:
                    epigrafe_nombre = epi.get("nombre")
                    epigrafe_identificador = epi.get("identificador")
                    items = epi.get("item")
                    if items is None:
                        continue
                    if isinstance(items, dict):
                        items = [items]
                    for item in items:
                        out.append({
                            "publicacion": publicacion,
                            "fecha_publicacion": fecha_publicacion,
                            "diario_numero": diario_numero,
                            "diario_identificador": diario_identificador,
                            "diario_pdf_url": diario_pdf_url,
                            "seccion_codigo": seccion_codigo,
                            "seccion_nombre": seccion_nombre,
                            "departamento_codigo": dept_codigo,
                            "departamento_nombre": dept_nombre,
                            "epigrafe_nombre": epigrafe_nombre,
                            "epigrafe_identificador": epigrafe_identificador,
                            "item_identificador": item.get("identificador"),
                            "item_titulo": item.get("titulo"),
                            "item_pdf_url": (item.get("url_pdf") or {}).get("texto"),
                        })
            else:
                for dept in possib_dept or []:
                    dept_codigo = dept.get("codigo","")
                    dept_nombre = dept.get("nombre","")

                    for epi in dept.get("epigrafe", []) or []:
                        epigrafe_nombre = epi.get("nombre")
                        epigrafe_identificador = epi.get("identificador")
                        items = epi.get("item")
                        if items is None:
                            continue
                        if isinstance(items, dict):
                            items = [items]
                        for item in items:
                            out.append({
                                "publicacion": publicacion,
                                "fecha_publicacion": fecha_publicacion,
                                "diario_numero": diario_numero,
                                "diario_identificador": diario_identificador,
                                "diario_pdf_url": diario_pdf_url,
                                "seccion_codigo": seccion_codigo,
                                "seccion_nombre": seccion_nombre,
                                "departamento_codigo": dept_codigo,
                                "departamento_nombre": dept_nombre,
                                "epigrafe_nombre": epigrafe_nombre,
                                "epigrafe_identificador": epigrafe_identificador,
                                "item_identificador": item.get("identificador"),
                                "item_titulo": item.get("titulo"),
                                "item_pdf_url": (item.get("url_pdf") or {}).get("texto"),
                            })
    return out
# --- end parser ---
