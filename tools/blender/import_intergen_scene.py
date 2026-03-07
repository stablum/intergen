import json
import os
import sys
import traceback

import bpy
from mathutils import Matrix, Vector

SUN_ENERGY_SCALE = 10000.0
POINT_ENERGY_SCALE = 4000.0


def main() -> None:
    snapshot_path, output_path = parse_args()
    with open(snapshot_path, "r", encoding="utf-8") as handle:
        snapshot = json.load(handle)

    ensure_parent_dir(output_path)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    configure_scene(scene, snapshot)

    collection = ensure_collection("Intergen")
    material_cache = {}
    for object_data in snapshot["objects"]:
        create_mesh_object(collection, object_data, material_cache)

    create_camera(collection, scene, snapshot["camera"])
    create_lights(collection, snapshot["lights"])
    build_compositor(scene, snapshot["evaluated_effects"])
    store_metadata(snapshot)

    bpy.ops.wm.save_as_mainfile(filepath=output_path)
    print(f"Saved Blender scene to {output_path}")


def parse_args() -> tuple[str, str]:
    if "--" not in sys.argv:
        raise RuntimeError("Expected blender importer arguments after '--'.")
    index = sys.argv.index("--")
    args = sys.argv[index + 1 :]
    if len(args) != 2:
        raise RuntimeError(
            "Usage: blender --background --python tools/blender/import_intergen_scene.py -- <snapshot.json> <output.blend>"
        )
    return args[0], args[1]


def ensure_parent_dir(path: str) -> None:
    parent = os.path.dirname(path)
    if parent:
        os.makedirs(parent, exist_ok=True)


def configure_scene(scene: bpy.types.Scene, snapshot: dict) -> None:
    state = snapshot.get("state", {})
    window = state.get("window", {})
    rendering = state.get("rendering", {})

    scene.render.resolution_x = int(window.get("width", 1440))
    scene.render.resolution_y = int(window.get("height", 960))
    scene.render.resolution_percentage = 100
    available_engines = {
        item.identifier for item in scene.render.bl_rna.properties["engine"].enum_items
    }
    if "BLENDER_EEVEE_NEXT" in available_engines:
        scene.render.engine = "BLENDER_EEVEE_NEXT"
    elif "BLENDER_EEVEE" in available_engines:
        scene.render.engine = "BLENDER_EEVEE"
    else:
        scene.render.engine = sorted(available_engines)[0]

    world = scene.world or bpy.data.worlds.new("IntergenWorld")
    scene.world = world
    world.use_nodes = True
    nodes = world.node_tree.nodes
    links = world.node_tree.links
    nodes.clear()

    background = nodes.new("ShaderNodeBackground")
    background.location = (0, 0)
    clear_color = rendering.get("clear_color", [0.0, 0.0, 0.0])
    background.inputs[0].default_value = (
        float(clear_color[0]),
        float(clear_color[1]),
        float(clear_color[2]),
        1.0,
    )
    background.inputs[1].default_value = 1.0

    output = nodes.new("ShaderNodeOutputWorld")
    output.location = (220, 0)
    links.new(background.outputs[0], output.inputs[0])


def ensure_collection(name: str) -> bpy.types.Collection:
    collection = bpy.data.collections.get(name)
    if collection is None:
        collection = bpy.data.collections.new(name)
        bpy.context.scene.collection.children.link(collection)
    return collection


def create_mesh_object(
    collection: bpy.types.Collection,
    object_data: dict,
    material_cache: dict[str, bpy.types.Material],
) -> None:
    mesh = bpy.data.meshes.new(object_data["name"])
    vertices = [tuple(vertex) for vertex in object_data["vertices"]]
    faces = [tuple(face) for face in object_data["faces"]]
    mesh.from_pydata(vertices, [], faces)
    mesh.validate(verbose=False)
    mesh.update()

    obj = bpy.data.objects.new(object_data["name"], mesh)
    collection.objects.link(obj)

    material = get_or_create_material(object_data["material"], material_cache)
    mesh.materials.append(material)

    obj["intergen_node_index"] = int(object_data["node_index"])
    obj["intergen_kind"] = object_data["kind"]
    obj["intergen_level"] = int(object_data["level"])
    if object_data.get("parent_index") is not None:
        obj["intergen_parent_index"] = int(object_data["parent_index"])
    if object_data.get("parent_vertex_index") is not None:
        obj["intergen_parent_vertex_index"] = int(object_data["parent_vertex_index"])


def get_or_create_material(
    material_data: dict,
    material_cache: dict[str, bpy.types.Material],
) -> bpy.types.Material:
    cache_key = json.dumps(material_data, sort_keys=True)
    if cache_key in material_cache:
        return material_cache[cache_key]

    material = bpy.data.materials.new(f"IntergenMaterial_{len(material_cache):03}")
    material.use_nodes = True
    node_tree = material.node_tree
    principled = node_tree.nodes.get("Principled BSDF")
    if principled is None:
        raise RuntimeError("Principled BSDF node was missing from the default material node tree.")

    base_color = tuple(material_data["base_color"])
    set_socket_default(principled, ["Base Color"], base_color)
    set_socket_default(principled, ["Metallic"], float(material_data["metallic"]))
    set_socket_default(principled, ["Roughness"], float(material_data["roughness"]))
    set_socket_default(
        principled,
        ["Specular IOR Level", "Specular"],
        float(material_data["reflectance"]),
    )
    set_socket_default(principled, ["Alpha"], float(material_data["opacity"]))

    if hasattr(material, "blend_method"):
        material.blend_method = "BLEND" if material_data["opacity"] < 0.999 else "OPAQUE"
    if hasattr(material, "shadow_method"):
        material.shadow_method = "HASHED" if material_data["opacity"] < 0.999 else "OPAQUE"

    material.diffuse_color = base_color
    material_cache[cache_key] = material
    return material


def set_socket_default(node: bpy.types.Node, names: list[str], value) -> None:
    for name in names:
        socket = node.inputs.get(name)
        if socket is not None:
            socket.default_value = value
            return


def create_camera(
    collection: bpy.types.Collection,
    scene: bpy.types.Scene,
    camera_data: dict,
) -> None:
    camera = bpy.data.cameras.new("IntergenCamera")
    obj = bpy.data.objects.new("IntergenCamera", camera)
    collection.objects.link(obj)
    obj.location = tuple(camera_data["position"])
    obj.rotation_mode = "QUATERNION"
    obj.rotation_quaternion = basis_quaternion(camera_data["forward"], camera_data["up"])
    scene.camera = obj


def create_lights(collection: bpy.types.Collection, lights_data: dict) -> None:
    directional = lights_data["directional"]
    directional_data = bpy.data.lights.new("IntergenSun", type="SUN")
    directional_data.color = tuple(directional["color"])
    directional_data.energy = max(0.001, float(directional["illuminance"]) / SUN_ENERGY_SCALE)
    if hasattr(directional_data, "use_shadow"):
        directional_data.use_shadow = bool(directional.get("shadows_enabled", True))
    directional_obj = bpy.data.objects.new("IntergenSun", directional_data)
    collection.objects.link(directional_obj)
    directional_obj.location = tuple(directional["position"])
    directional_obj.rotation_mode = "QUATERNION"
    directional_obj.rotation_quaternion = basis_quaternion(directional["forward"], (0.0, 0.0, 1.0))

    point = lights_data["point"]
    point_data = bpy.data.lights.new("IntergenPoint", type="POINT")
    point_data.color = tuple(point["color"])
    point_data.energy = max(0.001, float(point["intensity"]) / POINT_ENERGY_SCALE)
    if hasattr(point_data, "use_shadow"):
        point_data.use_shadow = bool(point.get("shadows_enabled", False))
    if hasattr(point_data, "use_custom_distance"):
        point_data.use_custom_distance = True
    if hasattr(point_data, "cutoff_distance"):
        point_data.cutoff_distance = max(0.1, float(point.get("range", 1.0)))
    if hasattr(point_data, "shadow_soft_size"):
        point_data.shadow_soft_size = max(0.05, float(point.get("range", 1.0)) * 0.03)
    point_obj = bpy.data.objects.new("IntergenPoint", point_data)
    collection.objects.link(point_obj)
    point_obj.location = tuple(point["position"])


def basis_quaternion(forward_raw, up_raw):
    forward = Vector(forward_raw)
    if forward.length < 1.0e-6:
        forward = Vector((0.0, -1.0, 0.0))
    forward.normalize()

    up = Vector(up_raw)
    if up.length < 1.0e-6:
        up = Vector((0.0, 0.0, 1.0))
    up.normalize()

    z_axis = (-forward).normalized()
    x_axis = up.cross(z_axis)
    if x_axis.length < 1.0e-6:
        fallback_up = Vector((0.0, 1.0, 0.0)) if abs(z_axis.z) > 0.99 else Vector((0.0, 0.0, 1.0))
        x_axis = fallback_up.cross(z_axis)
    x_axis.normalize()
    y_axis = z_axis.cross(x_axis).normalized()

    rotation_matrix = Matrix((x_axis, y_axis, z_axis)).transposed().to_4x4()
    return rotation_matrix.to_quaternion()


def build_compositor(scene: bpy.types.Scene, effects: dict) -> None:
    scene.use_nodes = True
    existing = scene.compositing_node_group
    if existing is not None and existing.users == 0:
        bpy.data.node_groups.remove(existing)
    node_tree = bpy.data.node_groups.new("IntergenCompositor", "CompositorNodeTree")
    scene.compositing_node_group = node_tree
    nodes = node_tree.nodes
    links = node_tree.links
    nodes.clear()

    render_layers = nodes.new("CompositorNodeRLayers")
    render_layers.location = (0, 0)
    current_output = render_layers.outputs["Image"]
    x_cursor = 220

    current_output, x_cursor = build_lens_distortion(node_tree, current_output, effects["lens_distortion"], x_cursor)
    current_output, x_cursor = build_wavefolder(node_tree, current_output, effects["color_wavefolder"], x_cursor)
    current_output, x_cursor = build_gaussian_blur(node_tree, current_output, effects["gaussian_blur"], x_cursor)
    current_output, x_cursor = build_bloom(node_tree, current_output, effects["bloom"], x_cursor)
    current_output, x_cursor = build_edge_detection(node_tree, current_output, effects["edge_detection"], x_cursor)

    node_tree.interface.new_socket("Image", in_out="OUTPUT", socket_type="NodeSocketColor")
    group_output = nodes.new("NodeGroupOutput")
    group_output.location = (x_cursor + 220, 0)
    links.new(current_output, group_output.inputs[0])


def build_lens_distortion(node_tree, source, config, x_cursor):
    if not config.get("enabled", False):
        return source, x_cursor

    nodes = node_tree.nodes
    links = node_tree.links
    lens = nodes.new("CompositorNodeLensdist")
    lens.location = (x_cursor, 0)
    fit_socket = lens.inputs.get("Fit")
    if fit_socket is not None:
        fit_socket.default_value = True
    distort_socket = lens.inputs.get("Distortion") or lens.inputs[2]
    distort_socket.default_value = (
        float(config.get("strength", 0.0))
        + float(config.get("radial_k2", 0.0)) * 0.35
        + float(config.get("radial_k3", 0.0)) * 0.15
    )
    dispersion_socket = lens.inputs.get("Dispersion") or lens.inputs[3]
    dispersion_socket.default_value = float(config.get("chromatic_aberration", 0.0))
    links.new(source, lens.inputs[0])
    return lens.outputs[0], x_cursor + 220


def build_wavefolder(node_tree, source, config, x_cursor):
    if not config.get("enabled", False):
        return source, x_cursor

    nodes = node_tree.nodes
    links = node_tree.links
    separate = nodes.new("CompositorNodeSeparateColor")
    separate.location = (x_cursor, 0)
    combine = nodes.new("CompositorNodeCombineColor")
    combine.location = (x_cursor + 360, 0)
    links.new(source, separate.inputs[0])

    gain = max(0.0, float(config.get("gain", 1.0)))
    modulus = max(0.0001, float(config.get("modulus", 1.0)))
    channel_offsets = {"Red": 140, "Green": 20, "Blue": -100}
    output_names = [("Red", 0), ("Green", 1), ("Blue", 2)]
    for channel_name, output_index in output_names:
        multiply = nodes.new("ShaderNodeMath")
        multiply.operation = "MULTIPLY"
        multiply.location = (x_cursor + 120, channel_offsets[channel_name])
        multiply.inputs[1].default_value = gain

        modulo = nodes.new("ShaderNodeMath")
        modulo.operation = "MODULO"
        modulo.location = (x_cursor + 240, channel_offsets[channel_name])
        modulo.inputs[1].default_value = modulus

        links.new(separate.outputs[channel_name], multiply.inputs[0])
        links.new(multiply.outputs[0], modulo.inputs[0])
        links.new(modulo.outputs[0], combine.inputs[output_index])

    links.new(separate.outputs["Alpha"], combine.inputs[3])
    return combine.outputs[0], x_cursor + 520


def build_gaussian_blur(node_tree, source, config, x_cursor):
    if not config.get("enabled", False):
        return source, x_cursor

    nodes = node_tree.nodes
    links = node_tree.links
    blur = nodes.new("CompositorNodeBlur")
    blur.location = (x_cursor, 0)
    blur.filter_type = "GAUSS"
    blur.use_relative = False
    radius = int(round(float(config.get("radius_pixels", 0)) * max(float(config.get("sigma", 1.0)), 0.5)))
    blur.size_x = max(1, radius)
    blur.size_y = max(1, radius)
    links.new(source, blur.inputs[0])
    return blur.outputs[0], x_cursor + 220


def build_bloom(node_tree, source, config, x_cursor):
    if not config.get("enabled", False):
        return source, x_cursor

    nodes = node_tree.nodes
    links = node_tree.links
    glare = nodes.new("CompositorNodeGlare")
    glare.location = (x_cursor, 0)
    type_socket = glare.inputs.get("Type")
    if type_socket is not None:
        type_socket.default_value = "Fog Glow"
    threshold_socket = glare.inputs.get("Threshold")
    if threshold_socket is not None:
        threshold_socket.default_value = float(config.get("threshold", 0.8))
    size_socket = glare.inputs.get("Size")
    if size_socket is not None:
        size_socket.default_value = float(min(9, max(1, int(round(float(config.get("radius_pixels", 8)) * 0.5)))))
    strength_socket = glare.inputs.get("Strength")
    if strength_socket is not None:
        strength_socket.default_value = float(config.get("intensity", 0.65))
    links.new(source, glare.inputs[0])
    return glare.outputs[0], x_cursor + 240


def build_edge_detection(node_tree, source, config, x_cursor):
    if not config.get("enabled", False):
        return source, x_cursor

    nodes = node_tree.nodes
    links = node_tree.links

    sobel = nodes.new("CompositorNodeFilter")
    sobel.filter_type = "SOBEL"
    sobel.location = (x_cursor, 0)
    links.new(source, sobel.inputs[0])

    rgb_to_bw = nodes.new("CompositorNodeRGBToBW")
    rgb_to_bw.location = (x_cursor + 150, 0)
    links.new(sobel.outputs[0], rgb_to_bw.inputs[0])

    multiply = nodes.new("ShaderNodeMath")
    multiply.operation = "MULTIPLY"
    multiply.location = (x_cursor + 300, 0)
    multiply.inputs[1].default_value = float(config.get("strength", 1.0))
    links.new(rgb_to_bw.outputs[0], multiply.inputs[0])

    subtract = nodes.new("ShaderNodeMath")
    subtract.operation = "SUBTRACT"
    subtract.location = (x_cursor + 440, 0)
    subtract.inputs[1].default_value = float(config.get("threshold", 0.0))
    links.new(multiply.outputs[0], subtract.inputs[0])

    clamp = nodes.new("ShaderNodeMath")
    clamp.operation = "MAXIMUM"
    clamp.location = (x_cursor + 580, 0)
    clamp.inputs[1].default_value = 0.0
    links.new(subtract.outputs[0], clamp.inputs[0])

    factor = nodes.new("ShaderNodeMath")
    factor.operation = "MULTIPLY"
    factor.use_clamp = True
    factor.location = (x_cursor + 720, 0)
    factor.inputs[1].default_value = float(config.get("mix", 1.0))
    links.new(clamp.outputs[0], factor.inputs[0])

    edge_color = nodes.new("CompositorNodeRGB")
    edge_color.location = (x_cursor + 720, -140)
    color = config.get("color", [1.0, 1.0, 1.0])
    edge_color.outputs[0].default_value = (float(color[0]), float(color[1]), float(color[2]), 1.0)

    mix = nodes.new("ShaderNodeMixRGB")
    mix.blend_type = "MIX"
    mix.location = (x_cursor + 920, 0)
    links.new(factor.outputs[0], mix.inputs[0])
    links.new(source, mix.inputs[1])
    links.new(edge_color.outputs[0], mix.inputs[2])
    return mix.outputs[0], x_cursor + 1140


def store_metadata(snapshot: dict) -> None:
    write_text_block("intergen_export.json", json.dumps(snapshot, indent=2, sort_keys=True))
    write_text_block("intergen_effects.json", json.dumps(snapshot.get("effects", {}), indent=2, sort_keys=True))
    write_text_block(
        "intergen_evaluated_effects.json",
        json.dumps(snapshot.get("evaluated_effects", {}), indent=2, sort_keys=True),
    )
    bpy.context.scene["intergen_export_format_version"] = int(snapshot.get("format_version", 1))
    bpy.context.scene["intergen_exported_at_unix_ms"] = str(snapshot.get("exported_at_unix_ms", 0))


def write_text_block(name: str, contents: str) -> None:
    existing = bpy.data.texts.get(name)
    if existing is not None:
        bpy.data.texts.remove(existing)
    text = bpy.data.texts.new(name)
    text.from_string(contents)


if __name__ == "__main__":
    try:
        main()
    except Exception as error:  # noqa: BLE001
        print(f"Intergen Blender import failed: {error}", file=sys.stderr)
        traceback.print_exc()
        raise











