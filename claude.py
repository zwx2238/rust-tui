import os
from anthropic import Anthropic

# 初始化客户端
client = Anthropic(
    api_key=os.environ.get("ANTHROPIC_API_KEY"),
)

# 发送消息
message = client.messages.create(
    max_tokens=1024,
    messages=[
        {
            "role": "user",
            "content": "Hello, Claude",
        }
    ],
    model="claude-sonnet-4-5-20250929",
)
print(message.content)
