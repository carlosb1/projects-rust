from dataclasses import dataclass

import spacy


def spacy_extract(text: str):
    import spacy
    nlp = spacy.load("es_core_news_md")
    doc = nlp(text)
    for ent in doc.ents:
        print(ent.text, ent.label_)

    for token in doc:
        if token.like_num:
            print(quantities(token.text))

def spacy_matcher(text: str):
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

def search_dates(text: str):
    from dateparser.search import search_dates
    dates = search_dates(text, languages=["es"])
    print(dates)

def quantities(text: str):
    from pint import UnitRegistry
    ureg = UnitRegistry()
    ureg.load_definitions("pint_es_aliases.txt")
    return ureg(text)