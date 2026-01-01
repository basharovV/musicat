<script lang="ts">
    import { onMount } from "svelte";
    import * as THREE from "three";
    import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
    import { EffectComposer } from "three/examples/jsm/postprocessing/EffectComposer.js";
    import { RenderPass } from "three/examples/jsm/postprocessing/RenderPass.js";
    import type { SongsByCountry } from "../../App";
    import { queueCountry } from "../../data/store";
    import { setQueue } from "../../data/storeHelper";
    import { locale } from "../../i18n/i18n-svelte";
    import { currentThemeObject } from "../../theming/store";
    import {
        getCountryColor,
        latLngToSphere,
        sphericalInterpolation,
        triangulatePolygon,
        xyzToLatLng,
    } from "../../utils/GeoUtils";
    import FakeGlowMaterial from "./FakeGlowMaterial";
    import Dropdown from "../ui/Dropdown.svelte";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";

    let globeContainer: HTMLDivElement;
    let scene: THREE.Scene;
    let composer: EffectComposer;
    let camera: THREE.PerspectiveCamera;
    let controls: OrbitControls;

    let renderer: THREE.WebGLRenderer;
    let raycaster = new THREE.Raycaster();
    let mouse = new THREE.Vector2();

    // Country selection state
    let countriesGroup = new THREE.Group();
    let hoveredCountryCode: string = "";
    export let selectedCountryName: string = "";

    let globe: THREE.Mesh;
    let clouds: THREE.Mesh;
    let glowSphere: THREE.Mesh;
    let width: number;
    let height: number;
    const radius = 7; // Increase sphere radius

    const markers: THREE.Sprite[] = [];
    const lines: THREE.Line[] = [];

    export let onCountryHovered: (code: string) => void;
    export let songData: SongsByCountry = {};

    async function initCountries() {
        const res = await fetch("/assets/ne_110m_admin_0_countries.geojson");
        const geojson = await res.json();
        const meshMat = new THREE.ShaderMaterial({
            uniforms: {
                color: {
                    value: new THREE.Color("transparent"),
                },
                fadeStrength: { value: 1.5 },
            },
            transparent: true,
            opacity: 0,
            side: selectedView === "mesh" ? THREE.DoubleSide : THREE.FrontSide,
            depthTest: true,
            depthWrite: false,
            polygonOffset: true,
            polygonOffsetFactor: 200,
            polygonOffsetUnits: 1,
            vertexShader: `
        varying vec3 vWorldPos;

        void main() {
            vec4 worldPos = modelMatrix * vec4(position, 1.0);
            vWorldPos = worldPos.xyz;
            gl_Position = projectionMatrix * viewMatrix * worldPos;
        }
    `,
            fragmentShader: `
        uniform vec3 color;
        uniform float fadeStrength;
        varying vec3 vWorldPos;

        void main() {
            vec3 viewDir = normalize(cameraPosition);
            vec3 normal = normalize(vWorldPos);

            float facing = dot(normal, viewDir);
			float fade = smoothstep(-0.2, 0.3, facing);
			fade = mix(0.15, 1.0, fade);
            fade = pow(fade, fadeStrength);
			vec3 finalColor = mix(color * 0.7, color, fade);

            gl_FragColor = vec4(finalColor, fade);
        }
    `,
        });

        const outlineMaterial = new THREE.ShaderMaterial({
            transparent: true,
            depthTest: true,
            depthWrite: false,
            uniforms: {
                color: {
                    value: new THREE.Color(
                        $currentThemeObject["mapview-globe-outline"],
                    ),
                },
                fadeStrength: { value: 1.5 },
            },
            vertexShader: `
        varying vec3 vWorldPos;

        void main() {
            vec4 worldPos = modelMatrix * vec4(position, 1.0);
            vWorldPos = worldPos.xyz;
            gl_Position = projectionMatrix * viewMatrix * worldPos;
        }
    `,
            fragmentShader: `
        uniform vec3 color;
        uniform float fadeStrength;
        varying vec3 vWorldPos;

        void main() {
            vec3 viewDir = normalize(cameraPosition);
            vec3 normal = normalize(vWorldPos);

            float facing = dot(normal, viewDir);
			float fade = smoothstep(-0.2, 0.3, facing);
			fade = mix(0.15, 1.0, fade);
            fade = pow(fade, fadeStrength);
			vec3 finalColor = mix(color * 0.7, color, fade);

            gl_FragColor = vec4(finalColor, fade);
        }
    `,
        });
        geojson.features.forEach((feature: any) => {
            const name = feature.properties[`NAME_${$locale.toUpperCase()}`];
            const { type, coordinates } = feature.geometry;
            const countryCode = feature.properties["ISO_A2_EH"];
            const polygons = type === "Polygon" ? [coordinates] : coordinates;

            polygons.forEach((poly: number[][][]) => {
                const [outer, ...holes] = poly;
                const geometry = triangulatePolygon(
                    outer,
                    holes,
                    radius * 1.007,
                );

                const mesh = new THREE.Mesh(geometry, meshMat.clone());
                // mesh.material.opacity = 0.6;
                const color = getCountryColor(
                    countryCode,
                    songData,
                    $currentThemeObject["mapview-scale-1"],
                    $currentThemeObject["mapview-scale-2"],
                );
                if (color) {
                    mesh.material.uniforms.color.value = color;
                    mesh.material.opacity = 1;
                    mesh.userData.nOfSongs =
                        songData[countryCode]?.data?.length;
                } else {
                    mesh.material.visible = false;
                }

                mesh.userData.countryCode = countryCode;
                mesh.userData.name = name;
                mesh.renderOrder = 1; // Add this - render countries after globe
                mesh.layers.set(0);

                // ---- Outline
                const outlinePoints = outer.map(([lng, lat]) =>
                    latLngToSphere(lat, lng, radius),
                );

                const lineGeo = new THREE.BufferGeometry().setFromPoints(
                    outlinePoints,
                );
                const line = new THREE.LineLoop(
                    lineGeo,
                    outlineMaterial.clone(),
                );
                line.renderOrder = 2; // Add this - render lines last
                line.castShadow = true;
                line.receiveShadow = true;
                line.layers.set(0);
                mesh.userData.line = line;

                countriesGroup.add(mesh);
                countriesGroup.add(line);
            });
        });

        scene.add(countriesGroup);
    }

    let isDragging = false;
    let downX = 0;
    let downY = 0;
    let isMouseDown = false;

    const DRAG_THRESHOLD = 5; // pixels

    function onMouseDown(event: MouseEvent) {
        downX = event.clientX;
        downY = event.clientY;
        isDragging = false;
        isMouseDown = true;
    }

    function onMouseMove(event: MouseEvent) {
        const dx = event.clientX - downX;
        const dy = event.clientY - downY;

        if (
            isMouseDown &&
            dx * dx + dy * dy > DRAG_THRESHOLD * DRAG_THRESHOLD
        ) {
            isDragging = true;
            tooltip?.classList.remove("active");
        }
        if (!isMouseDown) {
            handleInteractions(event); // hover
        }
    }

    function onMouseUp(event: MouseEvent) {
        if (!isDragging) {
            handleInteractions(event); // click
        }
        isDragging = false;
        isMouseDown = false;
        downX = 0;
        downY = 0;
    }

    export function zoomIn(amount = 1) {
        if (!camera || !controls) return;

        const dir = camera.position.clone().normalize();
        camera.position.addScaledVector(dir, -amount);
        controls.update();
    }

    export function zoomOut(amount = 1) {
        if (!camera || !controls) return;

        const dir = camera.position.clone().normalize();
        camera.position.addScaledVector(dir, amount);
        controls.update();
    }

    function handleInteractions(event: MouseEvent) {
        if (!renderer || !renderer.domElement || !globe) return;

        const canvas = renderer.domElement;
        const rect = canvas.getBoundingClientRect();

        mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
        mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

        raycaster.setFromCamera(mouse, camera);

        // First, intersect with the globe itself
        const globeIntersects = raycaster.intersectObject(globe);
        if (globeIntersects.length > 0) {
            // Get the intersection point on the globe surface
            const intersectionPoint = globeIntersects[0].point;

            // Convert the 3D point to latitude and longitude
            const { lat, lng } = xyzToLatLng(intersectionPoint);

            // Now find which country contains this lat/lng
            const country = findCountryAtCoordinates(lat, lng);
            if (country) {
                if (event.type === "mouseup") {
                    selectedCountryName = country.userData.name;
                    try {
                        playCountry();
                    } catch (err) {
                        console.error("error playing country", err);
                    }
                } else if (event.type === "mousemove") {
                    if (tooltip) {
                        // Update tooltip position
                        const container =
                            globeContainer.getBoundingClientRect();
                        const space = 5; // Space between the cursor and tooltip element

                        // Tooltip
                        const { height, width } =
                            tooltip.getBoundingClientRect();
                        const topIsPassed =
                            event.clientY <= container.top + height + space;
                        let top = event.pageY - space;
                        let left = event.pageX - space;

                        // Ensure the tooltip will never cross outside the canvas area(map)
                        if (topIsPassed) {
                            // Top:
                            top += height + space;

                            // The cursor is a bit larger from left side
                            left -= space * 2;
                        }

                        if (event.clientX < container.left + width) {
                            // Left:
                            left = event.pageX + space + 2;

                            if (topIsPassed) {
                                left += space * 2;
                            }
                        }

                        tooltip.style.top = `${top}px`;
                        tooltip.style.left = `${left}px`;
                    }
                    if (!country.userData.countryCode) {
                        tooltip?.classList.remove("active");
                        hoveredCountryCode = null;
                        return;
                    }

                    if (hoveredCountryCode !== country.userData.countryCode) {
                        onCountryHovered &&
                            onCountryHovered(country.userData.countryCode);
                        hoveredCountryCode = country.userData.countryCode;

                        if (!tooltip) return;

                        tooltip.classList.add("active");
                    }
                }
            } else {
                tooltip?.classList.remove("active");
                hoveredCountryCode = null;
            }
        } else {
            tooltip?.classList.remove("active");
            hoveredCountryCode = null;
            return;
        }
    }

    // Find which country mesh contains the given lat/lng coordinates
    function findCountryAtCoordinates(
        lat: number,
        lng: number,
    ): THREE.Mesh | null {
        // Create a small ray from the point outward to test intersection

        const origin = latLngToSphere(lat, lng, radius * 1.1); // outside
        const direction = origin.clone().normalize().negate(); // toward center

        // Test each country mesh to see if the point is inside it
        for (const child of countriesGroup.children) {
            if (child instanceof THREE.Mesh) {
                // Create a local raycaster from slightly inside the globe outward

                const ray = new THREE.Raycaster(origin, direction);

                const intersects = ray.intersectObject(child);

                // If we hit the mesh, this point is in this country
                if (intersects.length > 0) {
                    return child;
                }
            }
        }

        return null;
    }

    function playCountry() {
        if (!selectedCountryName) return;
        const allMeshes =
            (countriesGroup?.children?.filter((child) => {
                return child instanceof THREE.Mesh;
            }) as THREE.Mesh[]) || [];
        const countryMeshes = allMeshes.filter((child) => {
            return child.userData.name === selectedCountryName;
        }) as THREE.Mesh[];

        if (countryMeshes.length === 0) return;

        const code = countryMeshes[0]?.userData?.countryCode;

        if (!code) return;

        const songs = songData[code]?.data;
        if (!songs) return;

        // Play!
        $queueCountry = code;
        setQueue(songs, 0);
    }

    function setSelectedCountry(countryCode) {
        console.log("setSelectedCountry", countryCode);
        const allMeshes =
            (countriesGroup?.children?.filter((child) => {
                return child instanceof THREE.Mesh;
            }) as THREE.Mesh[]) || [];
        allMeshes?.forEach((child) => {
            const mat = child.material as THREE.ShaderMaterial;
            if (child.userData.countryCode === countryCode) {
                console.log("Setting selected country", countryCode);
                mat.visible = true;
                mat.uniforms.color.value = new THREE.Color(
                    $currentThemeObject["mapview-region-selected-bg"],
                );
            } else if (child.userData.nOfSongs === undefined) {
                mat.visible = false;
            } else if (child.userData.nOfSongs > 0) {
                const color = getCountryColor(
                    child.userData.countryCode,
                    songData,
                    $currentThemeObject["mapview-scale-1"],
                    $currentThemeObject["mapview-scale-2"],
                );
                mat.visible = true;
                mat.uniforms.color.value = new THREE.Color(color);
            }
        });
    }

    let isMounted = false;

    // Smoothly move the camera to focus on a specific marker
    function moveCameraToMarker(target: THREE.Vector3) {
        controls.autoRotate = false;
        // Animation variables
        const startPosition = camera.position.clone();
        console.log("navigating to", target);
        const cameraDistance = camera.position.length();

        const duration = 700; // Duration of the animation in milliseconds
        const startTime = performance.now();

        function animate() {
            const radius = 20; // Desired camera distance from the center of the globe
            const elapsed = performance.now() - startTime;
            let t = Math.min(elapsed / duration, 1);

            const interpolatedPoint = sphericalInterpolation(
                startPosition,
                target,
                t,
                cameraDistance,
            );
            camera.position.copy(interpolatedPoint);
            // console.log('interpolated', interpolatedPoint);
            // Update camera position
            // camera.position.setFromSpherical(spherical);

            // controls.target.lerpVectors(startTarget, new THREE.Vector3(0, 0, 0), t);

            // Update controls
            controls.update();

            // Render the scene
            composer.render();

            if (t < 1) {
                requestAnimationFrame(animate);
            }
        }

        animate();
    }

    let tooltip: HTMLDivElement;

    onMount(() => {
        setupGlobe();
        tooltip = document.querySelector("#map-tooltip");
        return () => {
            if (renderer && renderer.domElement) {
                renderer.domElement.removeEventListener(
                    "mousemove",
                    onMouseMove,
                );
                renderer.domElement.removeEventListener(
                    "mousedown",
                    onMouseDown,
                );
                renderer.domElement.removeEventListener("mouseup", onMouseUp);
            }
        };
    });

    $: if (isMounted && songData && $queueCountry) {
        console.log("TRIGGERED", $queueCountry);
        setSelectedCountry($queueCountry);

        // if (scene) scene.clear();
        markers.forEach((m) => {
            m.removeFromParent();
        });
        lines.forEach((l) => {
            l.removeFromParent();
        });
        addMarkers();
    }

    $: if (isMounted && selectedView === "satellite") {
        scene.remove(glowSphere);
        scene.add(globe);
        countriesGroup.children
            .filter((child) => {
                return child instanceof THREE.Mesh;
            })
            .forEach((child) => {
                const mat = child.material as THREE.ShaderMaterial;
                mat.side = THREE.FrontSide;
                mat.depthTest = false;
                mat.depthWrite = true;
            });
    } else if (isMounted && selectedView === "mesh") {
        scene.remove(globe);
        scene.add(glowSphere);
        countriesGroup.children
            .filter((child) => {
                return child instanceof THREE.Mesh;
            })
            .forEach((child) => {
                const mat = child.material as THREE.ShaderMaterial;
                mat.side = THREE.DoubleSide;
            });
    }

    async function setupGlobe() {
        // Get the width and height of the container
        width = globeContainer.clientWidth;
        height = globeContainer.clientHeight;

        // Create a Three.js scene
        scene = new THREE.Scene();
        camera = new THREE.PerspectiveCamera(60, width / height, 0.1, 1000);
        camera.position.z = 200; // Position the camera at a distance
        camera.layers.enable(0); // globe, countries (ignore clouds, glow, etc);
        camera.layers.enable(1); // globe, countries (ignore clouds, glow, etc);
        camera.layers.enable(2); // globe, countries (ignore clouds, glow, etc);
        renderer = new THREE.WebGLRenderer({
            alpha: true,
            antialias: true,
            powerPreference: "default",
            reversedDepthBuffer: true,
        });
        // renderer.setPixelRatio(window.devicePixelRatio * 1.5);
        renderer.setClearColor(0x000000, 0);
        renderer.shadowMap.enabled = true; // Enable shadow maps
        renderer.shadowMap.type = THREE.PCFSoftShadowMap;
        renderer.toneMapping = THREE.ACESFilmicToneMapping;
        renderer.outputColorSpace = THREE.SRGBColorSpace;
        renderer.setSize(width, height);
        globeContainer.appendChild(renderer.domElement);

        const renderPass = new RenderPass(scene, camera);
        // const filmPass = new FilmPass(0.5, false);
        composer = new EffectComposer(renderer);
        composer.addPass(renderPass);
        // composer.addPass(filmPass);

        // Create a raycaster and a mouse vector
        const raycaster = new THREE.Raycaster();
        const mouse = new THREE.Vector2();

        // Load the globe texture
        const textureLoader = new THREE.TextureLoader();
        const globeTexture = textureLoader.load("images/globe/earthmap4k.jpg");

        const globeMaterial = new THREE.MeshPhongMaterial({
            map: globeTexture,
            emissiveMap: textureLoader.load("images/globe/night.jpg"),
            bumpMap: textureLoader.load("images/globe/bumpmap.jpg"),
            emissive: new THREE.Color("#e5d4b2"),
            emissiveIntensity: 0.6,
            opacity: 1,
            specularMap: textureLoader.load("images/globe/specmap.jpg"),
            specular: 10,
            shininess: 10,
            bumpScale: 2,
        });

        const globeGeometry = new THREE.SphereGeometry(
            radius * 0.999,
            128,
            128,
        ); // Adjust radius accordingly
        globe = new THREE.Mesh(globeGeometry, globeMaterial);
        globe.castShadow = true; // Enable globe to cast shadows
        globe.receiveShadow = false; // Enable globe to receive shadows
        globe.renderOrder = 5; // Add this - render globe first
        globe.layers.set(0);
        if (selectedView === "satellite") {
            scene.add(globe);
        }

        const cloudsMat = new THREE.MeshStandardMaterial({
            map: textureLoader.load("images/globe/clouds.jpg"),
            transparent: true,
            blendColor: new THREE.Color("#242026"),
            opacity: 0.1,
        });

        clouds = new THREE.Mesh(globeGeometry, cloudsMat);
        clouds.layers.set(2);
        clouds.scale.setScalar(1.01);
        clouds.renderOrder = 2;
        // scene.add(clouds);

        // Create a glowing sphere using a custom material
        glowSphere = new THREE.Mesh(
            globeGeometry,
            new FakeGlowMaterial({
                glowColor: new THREE.Color(
                    $currentThemeObject["mapview-globe-glow"],
                ),
                glowInternalRadius: 0.01,
                falloff: 2,
                glowSharpness: 1,
                opacity: 0.035,
            }),
        );

        glowSphere.receiveShadow = true;
        glowSphere.castShadow = true;
        glowSphere.scale.multiplyScalar(1); // Scale up the glow sphere for effect
        glowSphere.layers.set(2);
        glowSphere.renderOrder = 2;
        if (selectedView === "mesh") {
            scene.add(glowSphere);
        }
        // scene.add(glowSphere);

        // Add a grid plane under the globe
        // const gridHelper = new THREE.GridHelper(1000, 20, new THREE.Color("#a2a2a2"), new THREE.Color("#cebed3")); // size and divisions
        // gridHelper.position.y = -100; // Position it under the globe
        // scene.add(gridHelper);

        // Optionally, add a helper to visualize the light's shadow camera
        // const helper = new THREE.CameraHelper(directionalLight.shadow.camera);
        // scene.add(helper);

        // Initial camera position
        camera.position.z = 18; // Adjust camera position for the larger globe
        // camera.position.set(14.415180352896348, 5.709076820917566, 4.2321724867717438);

        // Horizontal angle (azimuth) in radians
        const azimuth = Math.PI * -0.2; // Example: 45 degrees (in radians)
        // Vertical angle (altitude) in radians (maintain same vertical position)
        const altitude = Math.PI * 0.4; // Example: 45 degrees (in radians)
        let newPos = sphericalToCartesian(20, azimuth, altitude);
        camera.position.set(newPos.x, newPos.y, newPos.z);

        // Add a directional light to cast shadows
        const directionalLight = new THREE.DirectionalLight(0xffffff, 4);
        const lightPos = sphericalToCartesian(18, Math.PI * 0.1, Math.PI / 2);

        directionalLight.position.copy(lightPos); // Position the light at the camera's position
        directionalLight.castShadow = true; // Enable the light to cast shadows
        scene.add(directionalLight);

        // // Add a directional light to cast shadows
        const ambientLight = new THREE.AmbientLight(0xffffff, 0.5);
        // ambientLight.position.copy(camera.position); // Position the light at the camera's position
        ambientLight.castShadow = false; // Enable the light to cast shadows
        scene.add(ambientLight);

        // Adjust shadow map settings
        directionalLight.shadow.mapSize.width = 2048;
        directionalLight.shadow.mapSize.height = 2048;
        directionalLight.shadow.camera.near = 0.1;
        directionalLight.shadow.camera.far = 1000;

        // Add orbit controls for interactivity
        controls = new OrbitControls(camera, renderer.domElement);
        controls.enableDamping = true;
        controls.dampingFactor = 0.4; // Increase damping for smoother control
        controls.rotateSpeed = 0.2; // Increase rotation speed
        controls.autoRotate = true;
        controls.autoRotateSpeed = 0.5;
        controls.addEventListener("change", (ch) => {
            // directionalLight.position.copy(camera.position); // Position the light at the camera's position
        });

        // Add text label (initially invisible)
        const labelDiv = document.createElement("div");
        labelDiv.style.position = "absolute";
        labelDiv.style.backgroundColor = "rgba(255, 255, 255, 0.8)";
        labelDiv.style.padding = "4px";
        labelDiv.style.borderRadius = "4px";
        labelDiv.style.display = "none";
        document.body.appendChild(labelDiv);

        const animate = function () {
            requestAnimationFrame(animate);

            controls.update();
            // renderer.render(scene, camera);
            composer.render(); // Use composer for post-processing
            clouds.rotation.y += 0.00005;
        };

        animate();
        // Adjust canvas size on window resize
        window.addEventListener("resize", () => {
            camera.aspect =
                globeContainer.clientWidth / globeContainer.clientHeight;
            camera.updateProjectionMatrix();
            renderer.setSize(
                globeContainer.clientWidth,
                globeContainer.clientHeight,
            );
        });

        await initCountries();

        // ADD EVENT LISTENERS HERE - after everything is set up
        renderer.domElement.addEventListener("mousemove", onMouseMove);
        renderer.domElement.addEventListener("mousedown", onMouseDown);
        renderer.domElement.addEventListener("mouseup", onMouseUp);

        isMounted = true;
    }

    // Function to convert spherical coordinates to Cartesian coordinates
    function sphericalToCartesian(
        radius: number,
        azimuth: number,
        altitude: number,
    ): THREE.Vector3 {
        const x = radius * Math.cos(azimuth) * Math.sin(altitude);
        const y = radius * Math.cos(altitude);
        const z = radius * Math.sin(azimuth) * Math.sin(altitude);
        return new THREE.Vector3(x, y, z);
    }

    function addMarkers() {
        // Add markers
        const playing = new THREE.TextureLoader().load(
            "/images/globe/playing.png",
        );
        const allMeshes =
            (countriesGroup?.children?.filter((child) => {
                return child instanceof THREE.Mesh;
            }) as THREE.Mesh[]) || [];
        const countryMeshes = allMeshes.filter((child) => {
            return child.userData.countryCode === $queueCountry;
        });

        // Find the biggest mesh
        const biggestMesh = countryMeshes.reduce((a, b) => {
            const aArea = a.geometry.attributes.position.count / 3;
            const bArea = b.geometry.attributes.position.count / 3;
            return aArea > bArea ? a : b;
        });

        if (biggestMesh) {
            const boundingBox = new THREE.Box3().setFromObject(biggestMesh);
            const midpoint = new THREE.Vector3();
            boundingBox.getCenter(midpoint);
            const midpointCoordinates = xyzToLatLng(midpoint);
            // Load marker texture
            let map = playing;
            const spriteMaterial = new THREE.SpriteMaterial({
                map,
            });
            const marker = new THREE.Sprite(spriteMaterial);
            const [x, y, z] = convertLatLngToXYZ(
                midpointCoordinates.lat,
                midpointCoordinates.lng,
            );
            const markerPosition = new THREE.Vector3(x, y, z).multiplyScalar(
                1.25,
            ); // Pop out effect

            marker.position.copy(markerPosition);
            let scale = 0.6;
            marker.scale.set(scale, scale, scale); // Adjust scale if necessary
            marker.renderOrder = 5;
            markers.push(marker);
            scene.add(marker);

            // Create line from globe to marker
            const lineMaterial = new THREE.LineBasicMaterial({
                color: "#645960",
            });
            const points = [];
            points.push(new THREE.Vector3(x, y, z)); // Point on globe surface
            points.push(markerPosition); // Marker position

            const lineGeometry = new THREE.BufferGeometry().setFromPoints(
                points,
            );
            const line = createThickLineFromSurface(
                points[0],
                points[1].multiplyScalar(0.99),
                0.03,
                new THREE.Color($currentThemeObject["mapview-globe-outline"]),
            );

            line.renderOrder = 4;
            line.layers.set(1);
            lines.push(line);
            scene.add(line);
        }
    }

    function convertLatLngToXYZ(
        lat: number,
        lng: number,
    ): [number, number, number] {
        const phi = (90 - lat) * (Math.PI / 180);
        const theta = (lng + 180) * (Math.PI / 180);

        const x = -(radius * Math.sin(phi) * Math.cos(theta));
        const y = radius * Math.cos(phi);
        const z = radius * Math.sin(phi) * Math.sin(theta);

        return [x, y, z];
    }

    // Function to create a box representing a thick line from the globe surface point to the marker position
    function createThickLineFromSurface(
        pointA: THREE.Vector3,
        pointB: THREE.Vector3,
        thickness: number,
        color: THREE.Color,
    ): THREE.Mesh {
        const direction = new THREE.Vector3().subVectors(pointB, pointA);
        const length = direction.length();

        // Create a box geometry for the line
        const boxGeometry = new THREE.BoxGeometry(thickness, thickness, length);

        // Create a material for the box
        const boxMaterial = new THREE.MeshBasicMaterial({
            color: color,
            depthTest: true,
            depthWrite: false,
            transparent: true,
            side: THREE.DoubleSide,
        });

        // Create the mesh
        const box = new THREE.Mesh(boxGeometry, boxMaterial);

        // Position the box at the globe surface point
        box.position.copy(pointA);

        // Align the box with the direction of the line
        box.lookAt(pointB);

        // Since the default orientation of BoxGeometry is along the z-axis, we need to adjust the rotation
        const axis = new THREE.Vector3(0, 0, 1).cross(direction).normalize();
        const angle = Math.acos(
            direction.normalize().dot(new THREE.Vector3(0, 0, 1)),
        );
        box.setRotationFromAxisAngle(axis, angle);

        // Move the box so that its end touches the globe surface point
        box.translateZ(length / 2);

        return box;
    }

    let views = [
        {
            label: "Mesh",
            value: "mesh",
        },
        {
            label: "Satellite",
            value: "satellite",
        },
    ];

    export let selectedView = views[0].value;
</script>

<div
    id="globeContainer"
    bind:this={globeContainer}
    on:mouseenter={() => {
        if (controls) {
            controls.autoRotate = false;
            controls.update();
        }
    }}
    on:mouseleave={() => {
        if (controls) {
            controls.autoRotate = true;
            controls.update();
        }
    }}
></div>

<style lang="scss">
    #globeContainer {
        display: flex;
        justify-content: center;
        align-items: center;
        height: 100vh;
        width: 100%;
        overflow: hidden;
        cursor: grab;
        &:active {
            cursor: grabbing;
        }
        color: #242026;
    }

    .view-selector {
        position: absolute;
        display: flex;
        gap: 10px;
        top: 10px;
        right: 10px;
        z-index: 10;
    }
</style>
