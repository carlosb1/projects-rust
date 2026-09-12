import re


TARGET_CHARS = 2_000
MAX_CHARS = 3_000
OVERLAP_CHARS = 400


def create_chunks(
    text: str,
    target_chars: int = TARGET_CHARS,
    max_chars: int = MAX_CHARS,
    overlap_chars: int = OVERLAP_CHARS,
) -> list[str]:
    """
    Chunking genérico:

    1. Intenta respetar párrafos.
    2. Si un párrafo es enorme, intenta dividir por frases.
    3. Si una frase es enorme, divide por palabras.
    4. Nunca corta palabras por la mitad.
    5. Añade overlap entre chunks.
    """

    paragraphs = split_paragraphs(text)

    units: list[str] = []

    for paragraph in paragraphs:
        units.extend(
            split_long_paragraph(
                paragraph,
                max_chars=max_chars,
            )
        )

    return chunk_units(
        units,
        target_chars=target_chars,
        max_chars=max_chars,
        overlap_chars=overlap_chars,
    )


def split_paragraphs(text: str) -> list[str]:
    """
    Convierte texto extraído de PDF en párrafos razonables.
    """

    text = text.replace("\r\n", "\n").replace("\r", "\n")

    # Línea vacía = probable separación entre párrafos.
    raw_paragraphs = re.split(
        r"\n[ \t]*\n+",
        text,
    )

    paragraphs: list[str] = []

    for paragraph in raw_paragraphs:
        paragraph = paragraph.strip()

        if not paragraph:
            continue

        # Palabras cortadas por maquetación PDF:
        #
        # administra-
        # ciones
        #
        # ->
        #
        # administraciones
        paragraph = re.sub(
            r"(?<=\w)-[ \t]*\n[ \t]*(?=\w)",
            "",
            paragraph,
        )

        # Resto de saltos dentro del párrafo:
        #
        # "la presente resolución
        #  establece..."
        #
        # ->
        #
        # "la presente resolución establece..."
        paragraph = re.sub(
            r"[ \t]*\n[ \t]*",
            " ",
            paragraph,
        )

        # Normaliza espacios.
        paragraph = re.sub(
            r"[ \t]+",
            " ",
            paragraph,
        ).strip()

        if paragraph:
            paragraphs.append(paragraph)

    return paragraphs


def split_sentences(text: str) -> list[str]:
    """
    Sentence splitter sencillo y genérico.

    No se usa normalmente; solo cuando un párrafo supera MAX_CHARS.
    """

    pattern = r"(?<=[.!?])\s+(?=[A-ZÁÉÍÓÚÜÑ0-9])"

    return [
        sentence.strip()
        for sentence in re.split(pattern, text)
        if sentence.strip()
    ]


def split_long_paragraph(
    paragraph: str,
    max_chars: int,
) -> list[str]:
    """
    Divide únicamente párrafos demasiado grandes.

    Preferencia:

        párrafo
          ↓
        frases
          ↓
        palabras
    """

    if len(paragraph) <= max_chars:
        return [paragraph]

    sentences = split_sentences(paragraph)

    pieces: list[str] = []
    current: list[str] = []
    current_length = 0

    for sentence in sentences:

        # Una frase por sí sola supera el máximo.
        if len(sentence) > max_chars:

            if current:
                pieces.append(" ".join(current))
                current = []
                current_length = 0

            pieces.extend(
                split_by_words(
                    sentence,
                    max_chars=max_chars,
                )
            )

            continue

        additional_length = (
            len(sentence)
            + (1 if current else 0)
        )

        if (
            current
            and current_length + additional_length > max_chars
        ):
            pieces.append(" ".join(current))
            current = []
            current_length = 0

        current.append(sentence)

        current_length += (
            len(sentence)
            + (1 if len(current) > 1 else 0)
        )

    if current:
        pieces.append(" ".join(current))

    return pieces


def split_by_words(
    text: str,
    max_chars: int,
) -> list[str]:
    """
    Último recurso.

    Divide por espacios, nunca por posiciones arbitrarias de caracteres.
    Por tanto no parte palabras ni URLs.
    """

    words = text.split()

    pieces: list[str] = []
    current: list[str] = []
    current_length = 0

    for word in words:

        additional_length = (
            len(word)
            + (1 if current else 0)
        )

        if (
            current
            and current_length + additional_length > max_chars
        ):
            pieces.append(" ".join(current))
            current = []
            current_length = 0

        current.append(word)

        current_length += (
            len(word)
            + (1 if len(current) > 1 else 0)
        )

    if current:
        pieces.append(" ".join(current))

    return pieces


def chunk_units(
    units: list[str],
    target_chars: int,
    max_chars: int,
    overlap_chars: int,
) -> list[str]:
    """
    Agrupa párrafos/piezas en chunks.

    target_chars:
        tamaño deseado.

    max_chars:
        límite duro.

    overlap_chars:
        contexto reutilizado del chunk anterior.
    """

    chunks: list[str] = []

    current = ""
    has_new_content = False

    for unit in units:

        # Si ya hemos alcanzado aproximadamente el tamaño objetivo,
        # cerramos el chunk antes de introducir otra unidad.
        if (
            current
            and has_new_content
            and len(current) >= target_chars
        ):
            chunks.append(current.strip())

            current = get_overlap(
                current,
                overlap_chars,
            )

            has_new_content = False

        separator = "\n\n" if current else ""

        candidate = (
            current
            + separator
            + unit
        )

        # Aunque todavía no hayamos llegado al target,
        # nunca superamos MAX_CHARS.
        if (
            current
            and has_new_content
            and len(candidate) > max_chars
        ):
            chunks.append(current.strip())

            current = get_overlap(
                current,
                overlap_chars,
            )

            has_new_content = False

            separator = "\n\n" if current else ""

            candidate = (
                current
                + separator
                + unit
            )

        # Puede ocurrir que overlap + unidad > max.
        # En ese caso eliminamos overlap.
        if len(candidate) > max_chars:
            current = ""
            candidate = unit

        current = candidate
        has_new_content = True

    # Evitamos guardar un chunk compuesto únicamente de overlap.
    if current and has_new_content:
        chunks.append(current.strip())

    return chunks


def get_overlap(
    text: str,
    overlap_chars: int,
) -> str:
    """
    Obtiene aproximadamente los últimos N caracteres,
    pero sin cortar palabras.
    """

    if overlap_chars <= 0:
        return ""

    if len(text) <= overlap_chars:
        return text

    words = text.split()

    selected: list[str] = []
    current_length = 0

    for word in reversed(words):

        additional_length = (
            len(word)
            + (1 if selected else 0)
        )

        if (
            selected
            and current_length + additional_length > overlap_chars
        ):
            break

        selected.append(word)
        current_length += additional_length

    return " ".join(reversed(selected))