import re
from datetime import datetime

import pint
import spacy
from dateparser.search import search_dates
from pint import UndefinedUnitError
from spacy.matcher import Matcher


class MetadataExtractor:
    ENTITY_FIELDS = {
        "ORG": "organizations",
        "LOC": "locations",
        "PER": "persons",
        "MISC": "misc_entities",
    }

    def __init__(self, aliases_path: str | None = None):
        self.nlp = spacy.load("es_core_news_md")

        self.ureg = pint.UnitRegistry()
        if aliases_path:
            self.ureg.load_definitions(aliases_path)

        self.matcher = Matcher(self.nlp.vocab)
        self.matcher.add(
            "QUANTITY",
            [
                # 10 kilómetros, 2,5 kg, 30 °C
                [
                    {"LIKE_NUM": True},
                    {"ORTH": "°", "OP": "?"},
                    {"IS_ALPHA": True},
                ],
                # 25 €, 50 %
                [
                    {"LIKE_NUM": True},
                    {"ORTH": {"IN": ["€", "%"]}},
                ],
            ],
        )

    @staticmethod
    def _unique(values):
        return list(dict.fromkeys(values))

    @staticmethod
    def _normalize_number(text: str) -> str:
        # Convención española: 1.250,5 → 1250.5
        if "," in text:
            return text.replace(".", "").replace(",", ".")
        if re.fullmatch(r"[+-]?\d{1,3}(?:\.\d{3})+", text):
            return text.replace(".", "")
        return text

    def _extract_quantities(self, doc) -> list[dict]:
        result = []
        seen = set()

        for _, start, end in self.matcher(doc):
            span = doc[start:end]
            number = self._normalize_number(span[0].text)
            unit_text = "".join(token.text for token in span[1:])

            try:
                quantity = self.ureg.Quantity(float(number), unit_text)
            except (
                UndefinedUnitError,
                pint.errors.DefinitionSyntaxError,
                ValueError,
                TypeError,
            ):
                continue

            if quantity.dimensionless:
                continue

            value = float(quantity.magnitude)
            unit = str(quantity.units)
            key = (value, unit)

            if key in seen:
                continue

            seen.add(key)
            result.append({
                "text": span.text,
                "value": value,
                "unit": unit,
            })

        return result

    def _extract_dates(
        self,
        text: str,
        reference_date: datetime | None,
    ) -> list[dict]:
        # Sin referencia, solo acepta fechas completas.
        settings = {
            "STRICT_PARSING": reference_date is None,
            "RETURN_AS_TIMEZONE_AWARE": False,
        }
        if reference_date is not None:
            settings["RELATIVE_BASE"] = reference_date

        matches = search_dates(
            text,
            languages=["es"],
            settings=settings,
        ) or []

        result = []
        seen = set()

        for original_text, dt in matches:
            if not 1900 <= dt.year <= 2100:
                continue

            date = dt.date().isoformat()
            key = (original_text, date)

            if key not in seen:
                seen.add(key)
                result.append({
                    "text": original_text,
                    "value": date,
                })

        return result

    def extract(
        self,
        text: str,
        reference_date: datetime | None = None,
    ) -> dict:
        doc = self.nlp(text)

        payload = {
            "organizations": [],
            "locations": [],
            "persons": [],
            "misc_entities": [],
            "other_entities": [],
        }

        for entity in doc.ents:
            field = self.ENTITY_FIELDS.get(
                entity.label_,
                "other_entities",
            )
            value = entity.text.strip().rstrip(".")
            if value:
                payload[field].append(value)

        for field, values in payload.items():
            payload[field] = self._unique(values)

        date_mentions = self._extract_dates(text, reference_date)

        payload.update({
            "dates": self._unique(
                mention["value"] for mention in date_mentions
            ),
            "date_mentions": date_mentions,
            "quantities": self._extract_quantities(doc),
        })

        return payload


# Initialize one time
extractor = MetadataExtractor(
    aliases_path="pint_es_aliases.txt",
)


def build_payload(
    chunk_text: str,
    document_id: str,
    chunk_index: int,
    source: str,
    reference_date: datetime | None = None,
) -> dict:
    return {
        "text": chunk_text,
        "document_id": document_id,
        "chunk_index": chunk_index,
        "source": source,
        "metadata_version": 1,
        **extractor.extract(
            chunk_text,
            reference_date=reference_date,
        ),
    }


