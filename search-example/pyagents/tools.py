import re
def build_embedding_text(payload: dict) -> str:
    parts = []

    if payload.get("item_titulo"):
        parts.append(f"Título: {payload['item_titulo']}")

    if payload.get("departamento_nombre"):
        parts.append(
            f"Departamento: {payload['departamento_nombre']}"
        )

    if payload.get("seccion_nombre"):
        parts.append(f"Sección: {payload['seccion_nombre']}")

    if payload.get("epigrafe_nombre"):
        parts.append(f"Epígrafe: {payload['epigrafe_nombre']}")

    parts.append(f"Contenido:\n{payload['text']}")

    return "\n\n".join(parts)


#def chunk_text(text: str, chunk_chars: int = 1000, overlap: int = 200) -> List[str]:
#    if not text:
#        return []
#    chunks = []
#    step = max(1, chunk_chars - overlap)
#    for start in range(0, len(text), step):
#        chunk = text[start:start + chunk_chars]
#        if chunk.strip():
#            chunks.append(chunk)
#    return chunks


def create_chunks(
    text: str,
    max_chars: int = 2_500,
) -> list[str]:
    paragraphs = split_paragraphs(text)

    normalized_paragraphs: list[str] = []

    for paragraph in paragraphs:
        normalized_paragraphs.extend(
            split_long_paragraph(
                paragraph,
                max_chars=max_chars,
            )
        )

    return chunk_paragraphs(
        normalized_paragraphs,
        max_chars=max_chars,
    )

def split_paragraphs(text: str) -> list[str]:
    # Normaliza finales de línea.
    text = text.replace("\r\n", "\n").replace("\r", "\n")

    # Separa cuando existe al menos una línea en blanco.
    raw_paragraphs = re.split(r"\n\s*\n+", text)

    paragraphs = []

    for paragraph in raw_paragraphs:
        # Une los saltos de línea introducidos por el ancho del PDF.
        paragraph = re.sub(r"\s*\n\s*", " ", paragraph)

        # Normaliza espacios.
        paragraph = re.sub(r"[ \t]+", " ", paragraph).strip()

        if paragraph:
            paragraphs.append(paragraph)

    return paragraphs


def chunk_paragraphs(
    paragraphs: list[str],
    max_chars: int = 2_500,
) -> list[str]:
    chunks: list[str] = []
    current_paragraphs: list[str] = []
    current_length = 0

    for paragraph in paragraphs:
        additional_length = len(paragraph)

        if current_paragraphs:
            additional_length += 2  # "\n\n"

        if (
            current_paragraphs
            and current_length + additional_length > max_chars
        ):
            chunks.append("\n\n".join(current_paragraphs))
            current_paragraphs = []
            current_length = 0

        current_paragraphs.append(paragraph)
        current_length += additional_length

    if current_paragraphs:
        chunks.append("\n\n".join(current_paragraphs))

    return chunks

def split_sentences(text: str) -> list[str]:
    sentence_pattern = r"(?<=[.!?])\s+(?=[A-ZÁÉÍÓÚÜÑ])"

    return [
        sentence.strip()
        for sentence in re.split(sentence_pattern, text)
        if sentence.strip()
    ]

def split_long_paragraph(
    paragraph: str,
    max_chars: int,
) -> list[str]:
    if len(paragraph) <= max_chars:
        return [paragraph]

    sentences = split_sentences(paragraph)

    pieces: list[str] = []
    current: list[str] = []
    current_length = 0

    for sentence in sentences:
        # Una frase por sí sola supera el límite.
        if len(sentence) > max_chars:
            if current:
                pieces.append(" ".join(current))
                current = []
                current_length = 0

            for start in range(0, len(sentence), max_chars):
                pieces.append(sentence[start : start + max_chars])

            continue

        additional_length = len(sentence) + (1 if current else 0)

        if current and current_length + additional_length > max_chars:
            pieces.append(" ".join(current))
            current = []
            current_length = 0

        current.append(sentence)
        current_length += additional_length

    if current:
        pieces.append(" ".join(current))

    return pieces



