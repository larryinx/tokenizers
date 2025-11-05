from tokenizers import Tokenizer
from tokenizers.pre_tokenizers import CodeLexer, Sequence, ByteLevel

# Load a base tokenizer (or create one)
tokenizer = Tokenizer.from_file("./tokenizer.json")

# Create CodeLexer
code_lexer = CodeLexer(languages=["python", "py"])

# Chain with ByteLevel to preserve whitespace
byte_level = ByteLevel(add_prefix_space=False, trim_offsets=True, use_regex=False)
hybrid_pretok = Sequence([code_lexer, byte_level])

# Set the pre-tokenizer
tokenizer.pre_tokenizer = hybrid_pretok

text = """
```python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)
```
"""

# Now encode with the tokenizer
encoding = tokenizer.encode(text)
print(f"Tokens: {encoding.tokens}")
print(f"Token IDs: {encoding.ids}")

# Decode
decoded = tokenizer.decode(encoding.ids)
print(f"Decoded: {decoded}")
