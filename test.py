from transformers import PreTrainedTokenizerFast

tokenizer = PreTrainedTokenizerFast(tokenizer_file="./tokenizer.json")

# text = """
# Here is Python code:
# ```python
# def fibonacci(n):
#     if n <= 1:
#         return n

#     return fibonacci(n - 1) + fibonacci(n - 2)

# ```
# Done!

# Another line of text.

# ```python
# def fibonacci(n):
#     if n <= 1:
#         return n

#     return fibonacci(n - 1) + fibonacci(n - 2)
# ```

# Done!
# """

text = """You are given a Python function and an assertion containing an input to the function. Complete the assertion with a literal (no unsimplified expressions, no function calls) containing the output when executing the provided code on the given input, even if the function is incorrect or incomplete. Do NOT output any extra information. Provide the full assertion with the correct output in [ANSWER] and [/ANSWER] tags, following the examples.

```python
def repeatNumber(number : int) -> int:
    return number
```
assert repeatNumber(number = 17) == ??

[ANSWER]
assert repeatNumber(number = 17) == 17
[/ANSWER]

```python
def addCharacterA(string : str) -> str:
    return string + "a"
```
assert addCharacterA(string = "x9j") == ??

[ANSWER]
assert addCharacterA(string = "x9j") == "x9ja"
[/ANSWER]
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