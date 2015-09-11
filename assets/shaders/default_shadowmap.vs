#version 440

// uniform buffer bindings:
// 0: scene and pass global (includes lights)
// 1: per-material data
// 2: per-object data

layout (std140, binding = 0) uniform SceneData {
	mat4 viewMatrix;
	mat4 projMatrix;
	mat4 viewProjMatrix;	// = projMatrix*viewMatrix
	vec4 lightDir;
	vec4 wEye;	// in world space
	vec2 viewportSize;	// taille de la fenêtre
	vec3 wLightPos;
	vec3 lightColor;
	float lightIntensity;
};

layout (std140, binding = 1) uniform ObjectData {
	mat4 modelMatrix;
};

layout (std140, binding = 2) uniform LightData {
	mat4 shadowDepthMatrix;
};

// texture bindings:
// 0-3: pass textures (shadow maps, etc.)
// 4-8: per-material textures
// 8-?: per-object textures

layout(location=0) in vec3 position;
layout(location=1) in vec3 normal;
layout(location=2) in vec3 tangent;
layout(location=3) in vec2 texcoords;

out vec2 tc;
out vec3 wPos;
out vec3 wN;
out vec4 shadowCoords;

void main() {
	vec4 wPos_tmp = modelMatrix * vec4(position, 1.0);
	// TODO normal matrix
	vec4 wN_tmp = modelMatrix * vec4(normal, 0.0);
	vec4 temp_pos = projMatrix * viewMatrix * wPos_tmp;
	gl_Position = temp_pos;
	shadowCoords = shadowDepthMatrix * wPos_tmp;
	tc = texcoords;
	wPos = wPos_tmp.xyz;
	wN = wN_tmp.xyz;
}
