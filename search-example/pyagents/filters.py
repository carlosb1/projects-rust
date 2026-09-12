from dataclasses import dataclass

import spacy
from pint import UndefinedUnitError


def entities_to_payload(entities):
    payload = {
        "organizations": [],
        "locations": [],
        "persons": [],
        "misc_entities": [],
        "other_entities": [],
        "units": [],
    }

    mapping = {
        "ORG": "organizations",
        "LOC": "locations",
        "PER": "persons",
        "MISC": "misc_entities",
        "UNITS": "units",
    }

    for value, entity_type in entities:
        field = mapping.get(entity_type, "other_entities")
        if field:
            value = value.strip().rstrip(".")
            payload[field].append(value)

    # eliminar duplicados
    for field in payload:
        payload[field] = list(dict.fromkeys(payload[field]))

    return payload


def spacy_extract(text: str):
    import spacy
    nlp = spacy.load("es_core_news_md")
    doc = nlp(text)
    entities = []
    for ent in doc.ents:
        entities.append((ent.text, ent.label_))

    for token in doc:
        if token.like_num:
            possible_quant = quantities(token.text)
            if possible_quant:
                entities.append((token.text, "UNITS"))

    return entities_to_payload(entities)

def _spacy_matcher(text: str):
    from spacy.matcher import Matcher
    import spacy
    nlp = spacy.load("es_core_news_md")
    doc = nlp(text)

    matcher = Matcher(nlp.vocab)

    matcher.add(
        "MONEY",
        [[
            {"LIKE_NUM": True},
            {"LOWER": {"IN": ["euro", "euros", "€"]}}
        ]]
    )
    matcher.add(
        "DATE",
        [[
            {"LIKE_NUM": True},
            {"LOWER": "de"},
            {"LOWER": {
                "IN": [
                    "enero", "febrero", "marzo", "abril",
                    "mayo", "junio", "julio", "agosto",
                    "septiembre", "octubre", "noviembre", "diciembre"
                ]
            }},
            {"LOWER": "de"},
            {"LIKE_NUM": True}
        ]]
    )

    matches = matcher(doc)

    for match_id, start, end in matches:
        span = doc[start:end]
        print(span.text)


def dates_to_payload(dates):
    result = []

    for original_text, dt in dates:
        # evita cosas absurdas como año 3822, 0033, etc.
        if not 1900 <= dt.year <= 2100:
            continue
        result.append(dt.date().isoformat())

    return list(dict.fromkeys(result))

def search_dates(text: str):
    from dateparser.search import search_dates
    dates = search_dates(text, languages=["es"])
    return dates_to_payload(dates)

def quantities(text: str):
    from pint import UnitRegistry
    import pint
    ureg = UnitRegistry()
    ureg.load_definitions("pint_es_aliases.txt")
    try:
        val = ureg(text)
    except (UndefinedUnitError, ValueError):
        return None

    if not isinstance(val, pint.Quantity):
        return None
    if val.dimensionless:
        return None
    return val