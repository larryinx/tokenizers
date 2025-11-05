"""
Test script for CodeLexer with Llama 3.1 8B Instruct tokenizer
"""

from transformers import AutoTokenizer
from tokenizers import Tokenizer

# Load Llama 3.1 8B Instruct tokenizer
print("Loading Llama 3.1 8B Instruct tokenizer...")
tokenizer = AutoTokenizer.from_pretrained("meta-llama/Meta-Llama-3.1-8B-Instruct")

print(f"Original tokenizer type: {type(tokenizer)}")
print(f"Original tokenizer backend: {type(tokenizer.backend_tokenizer)}")

# Test with a conversation that includes code
conversation = [
    {
        "role": "user",
        "content": "Can you write a fibonacci function in Python?"
    },
    {
        "role": "assistant",
        "content": """Sure! Here's a fibonacci function:

```python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)
```

This is a recursive implementation."""
    }
]

print("\n" + "="*80)
print("Testing apply_chat_template with Llama 3.1 8B Instruct tokenizer")
print("="*80)

# Apply chat template
formatted_text = tokenizer.apply_chat_template(
    conversation
)

print("\nFormatted chat template:")
print(formatted_text)
print("\n" + "="*80)

print("Tokens:")
tokens = tokenizer.convert_ids_to_tokens(formatted_text)
print(tokens)

print("Decoding formatted text...")
decoded = tokenizer.decode(formatted_text)
print(f"Decoded: {decoded}")


hybrid_tokenizer = Tokenizer.from_file("./tokenizer.json")
print(f"Hybrid tokenizer type: {type(hybrid_tokenizer)}")

tokenizer.backend_tokenizer.pre_tokenizer = hybrid_tokenizer.pre_tokenizer


print("\n" + "="*80)
print("Testing apply_chat_template with CodeLexer")
print("="*80)

# Apply chat template
formatted_text = tokenizer.apply_chat_template(
    conversation
)

print("\nFormatted chat template:")
print(formatted_text)
print("\n" + "="*80)

print("Tokens:")
tokens = tokenizer.convert_ids_to_tokens(formatted_text)
print(tokens)

print("Decoding formatted text...")
decoded = tokenizer.decode(formatted_text)
print(f"Decoded: {decoded}")