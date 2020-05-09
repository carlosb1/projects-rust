from transformers import AutoTokenizer, AutoConfig
from transformers import AutoModelForSequenceClassification
from transformers import TextClassificationPipeline


def model_fn(name_model):
    tokenizer = AutoTokenizer.from_pretrained(name_model)
    model = AutoModelForSequenceClassification.from_pretrained(name_model)
    return model, tokenizer


def predict_fn(input_data, model):
    trained_model, tokenizer = model
    pipe = TextClassificationPipeline(model=trained_model, tokenizer=tokenizer)
    output = pipe(input_data)
    return output


SENTIMENT_MODEL = 'nlptown/bert-base-multilingual-uncased-sentiment'


class MyBertTransformerSentimentAnalysis():
    def __init__(self, name_model: str = SENTIMENT_MODEL):
        self.model_tuple = model_fn(name_model)

    def run(self, input_data: str) -> dict:
        predict_fn(input_data, self.model_tuple)
