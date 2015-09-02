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

in vec3 position;
out vec2 tc;
void main() {
	vec4 temp_pos = projMatrix * viewMatrix * vec4(position, 1.0);
	gl_Position = temp_pos;
	//gl_Position = vec4(position.xy, 0.0, 1.0);
	tc = temp_pos.xy;
}