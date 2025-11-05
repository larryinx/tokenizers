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

```python
abcdefg
```

```python
def repeatNumber(number : int) -> int:
    return number
assert repeatNumber(number = 17) == ??
```

Done!
"""

encoded = tokenizer(text)
print(f"Tokens: {encoded.tokens()}")
print(f"Token IDs: {encoded.input_ids}")

decoded = tokenizer.decode(encoded.input_ids)
print(f"Decoded: {decoded}")