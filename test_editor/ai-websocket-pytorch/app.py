#!/usr/bin/env python

import asyncio
from transformers.pipelines import token_classification
from websockets.server import serve
import json
import traceback

import torch
from transformers import BertTokenizerFast, EncoderDecoderModel
from torch.multiprocessing import set_start_method, Queue, Process

try:
     set_start_method('spawn', force=True)
except RuntimeError:
    pass

def process_tasks(task_queue, results_queue, summ_ai):
    while True:
        print("waiting for tasks")
        try:
            parsed = task_queue.get()
            print(f"{parsed}")
            text = parsed['value']
            identifier = parsed['identifier']
            #results_queue.put({"result": text, "identifier": identifier})
            print(f"starting summarization process")
            summarization = generate_summary(text, summ_ai)
            print(f"finished sum={summarization}")
            results_queue.put({"value": summarization,"typ": "text" , "identifier": identifier})
        except KeyboardInterrupt:
            print('interrupted and leaving!')
            return
        except:
            print(traceback.format_exc())
            return

# AI function
def generate_summary(text, summ_ai):
   (tokenizer, model, device) = summ_ai
   inputs = tokenizer([text], padding="max_length", truncation=True, max_length=512, return_tensors="pt")
   input_ids = inputs.input_ids.to(device)
   attention_mask = inputs.attention_mask.to(device)
   output = model.generate(input_ids, attention_mask=attention_mask)
   return tokenizer.decode(output[0], skip_special_tokens=True)

async def echo(websocket):
    async for new_content in websocket:
        #response = generate_summary(message)
        #await websocket.send(response)
        new_content = json.loads(new_content)
        print(new_content)
        #message = message + new_content["value"]
        #new_content["value"] = message

        task_queue.put(new_content)
        print(f"I am receiving this message {new_content['value']}")
        #TODO return a json message
        if not results_queue.empty():
            try:
                await websocket.send(json.dumps(results_queue.get_nowait()))
            except:
                print("Error getting value")
        else:
            print("Not available values")
            await websocket.send(json.dumps(new_content))

async def main():
    print("starting another websocket, why")
    async with serve(echo, "0.0.0.0", 8765):
        await asyncio.Future()  # run forever

if __name__ == '__main__':
    task_queue = Queue()
    results_queue = Queue()
    last_idenfifier = 0

    # init process
    device = 'cuda' if torch.cuda.is_available() else 'cpu'
    ckpt = 'mrm8488/bert2bert_shared-spanish-finetuned-summarization'
    tokenizer = BertTokenizerFast.from_pretrained(ckpt)
    model = EncoderDecoderModel.from_pretrained(ckpt).to(device)

    p = Process(target=process_tasks, args=(task_queue,results_queue, (tokenizer, model, device),  ))
    p.start()
    asyncio.run(main())

