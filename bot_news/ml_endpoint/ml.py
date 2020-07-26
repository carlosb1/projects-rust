from transformers import AutoTokenizer, AutoConfig
from transformers import AutoModelForSequenceClassification
from transformers import TextClassificationPipeline


SENTIMENT_MODEL = 'nlptown/bert-base-multilingual-uncased-sentiment'


class MyBertTransformerSentimentAnalysis():
    def __init__(self, name_model: str = SENTIMENT_MODEL):
        tokenizer = AutoTokenizer.from_pretrained(name_model)
        model = AutoModelForSequenceClassification.from_pretrained(name_model)
        self.pipe = TextClassificationPipeline(
            model=model, tokenizer=tokenizer)

    def run(self, input_data: str) -> dict:
        output = self.pipe(input_data)
        return output
