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
};

layout (std140, binding = 1) uniform MaterialData
{
	vec3 uColor;
};


// texture bindings:
// 0-3: pass textures (shadow maps, etc.)
// 4-8: per-material textures
// 8-?: per-object textures

layout (binding = 0) uniform sampler2D tex0;
in vec2 tc;
out vec4 color;
void main() {
	color = vec4(uColor, 1.0f);
}