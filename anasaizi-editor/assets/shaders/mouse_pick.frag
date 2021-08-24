#version 450

layout(location = 0) in vec4 objectId;

layout(location = 0) out vec4 out_objectId;

void main() {
    out_objectId = objectId;
}