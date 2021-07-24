Hobby game-engine project using vulkan and QT for UI.
If your here to look for code quality then your on the wrong place. 
Its mainly drafting something together and learning my way through vulkan. 
Vulkan is new for me so the code is in continue development. 


todo: 
cleanup resources:
- `vkDestroyDescriptorSetLayout(device, descriptorSetLayout, nullptr);`
- `VertexBuffer` and `IndexBuffer` memory and buffer.

Features:
- Reresh logic for uniform buffers, make sure to provide newest data too all images in swapchain.
 But dont update uniform buffers if not required. 