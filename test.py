from transformers import PreTrainedTokenizerFast

tokenizer = PreTrainedTokenizerFast(tokenizer_file="./tokenizer.json")

text = """
Here is Python code:
```python
def fibonacci(n):
    if n <= 1:
        return n

    return fibonacci(n - 1) + fibonacci(n - 2)

```
Done!

Another line of text.

```python
def fibonacci(n):
    if n <= 1:
        return n

    return fibonacci(n - 1) + fibonacci(n - 2)
```

Done!
"""


encoded = tokenizer(text)
print(f"Tokens: {encoded.tokens()}")
print(f"Token IDs: {encoded.input_ids}")

decoded = tokenizer.decode(encoded.input_ids)
print(f"Decoded: {decoded}")


from transformers import AutoTokenizer

original_tokenizer = AutoTokenizer.from_pretrained("meta-llama/Meta-Llama-3-8B-Instruct")

encoded = original_tokenizer(text)
print(f"Tokens: {encoded.tokens()}")
print(f"Token IDs: {encoded.input_ids}")

decoded = original_tokenizer.decode(encoded.input_ids)
print(f"Decoded: {decoded}")