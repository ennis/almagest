#version 440

layout (std140, binding = 0) uniform LightData {
	mat4 lightTransformMatrix;
	mat4 modelMatrix;
};

// texture bindings:
// 0-3: pass textures (shadow maps, etc.)
// 4-8: per-material textures
// 8-?: per-object textures

layout(location=0) in vec3 position;
layout(location=1) in vec3 normal;	// unused
layout(location=2) in vec3 tangent;
layout(location=3) in vec2 texcoords;

void main() {
	vec4 wPos = modelMatrix * vec4(position, 1.0);
	gl_Position = lightTransformMatrix * wPos;
}
