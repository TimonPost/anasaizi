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
- Staging to GPU buffer abstraction.
- Single command operation abstraction
- Combine index/vertex buffer into one structure
- create to new
- Make pipeline more generic.

TODO:

- Maak uniform objecten push constants werkend
- Maak uniform buffer update werkend.
- Test de nieuwe shader IO. 
 