There've always been some hurdles to overcome in order to accurately
simulate visual elements in computer graphics. Though, it goes without
saying that fire and water are among the most difficult to replicate,
not only because of their degree of unpredictability regarding fluid
dynamics, but also because of the actual physics of light involved
depending on which rendering technology comes into play.

It's with this in mind, and the importance of ocean physics to Abyssal
and its simulations of real-life scenarios that made my task as a
technical artist a necessary one, to further enhance the ocean system,
features, and looks of our software. Let me take you through a few of
the steps I took as well as give you some insight into how we got our
ocean looking as sleek as it does.

### Start with triangles (loads of them)

To simulate the ocean fluidity to the standard we wanted, we need to use
a full tessellated mesh. And even though the vertex shader rendering
process is expensive, it is still far lighter than a standard high
polygon mesh for the GPU due to how the subdivided triangles are
generated and subsequently interpolated using a *shader tessellation*.
And so I opted to create a distinction in the subdivision ratio of
tessellated triangles near and far from the camera as a way to further
optimize its impact on the GPU. This way we could greatly reduce the
amount of *vertex per pixel*, allowing for considerably faster
rendering.

![](refs/abyssal/assets/63b69f48c5c51c54ec49579ed04e1b04cc6159cf.png)

A distance function allows the ocean mesh to tessellate only in regions
where the camera is near to its surface.

At this point, we can start adding wave functions to the vertex height
information -- beginning with gravity waves. *Gravity waves*, which are
caused by the pressure of gravity, are more or less a simple swaying
effect of the tessellated geometry normals that create a background
noise for the actual bigger waves that we call *Trochoidal waves*.

![](refs/abyssal/assets/07c7c7bd97b5a1846b148a4cb9ddc3ae5e7bcce6.jpg)

The more Gerstner waves we add, the better the realness.

For the Trochoidal waves we use an approximation to the *Gerstner wave*
formula, which is standard use in computer graphics as it simulates the
nonlinear and unpredictable nature of a fluid's behavior rather well.

To increase entropy we needed to add more than a single Gerstner wave,
keeping in mind that the more waves that are added, the greater the
approximation to reality -- though at a cost: the more we add, the more
expensive they become in terms of GPU computation. And for that reason,
three waves seems to be the Goldilocks number.

Because we can then set up the function variables as we please to alter
the *Length*, *Frequency*, *Amplitude*, and *Steepness* of these waves
individually, we are able to greatly customize for any given situation
and atmospheric scenario.

![](refs/abyssal/assets/bdc3468a021287f82eff78e1e835cac565182a8f.jpg)
![](refs/abyssal/assets/212d23d3e108972ef05b97e0c592e5eca220d2cf.jpg)

Low Steepness -- High Steepness

After managing the height mapping information for the ocean mesh we
delve deeper into the shader works and start worrying about the visual
challenges of creating realistic water.

### Shader or Sorcery?

As I started the visual enhancement task I was immediately taken back by
a series of rendering limitations regarding reflections and lighting
consistencies and so a number of manual adjustments would have to be
made within the shader itself. Though, I'll get to that later -- first
let me talk a little about the basics that visually define water --
transmission and reflection.

Because water is *Translucent*, light that goes through it isn't blocked
as easily as light through an opaque surface. On top of that, there are
numerous variables that change its appearance, such as water
temperature, salinity, density, and so on. But in general how we
perceive it comes down to the angle in which light rays hit the surface
in reference to the human eye. This duality between light transmission
and reflection is what causes our eyes to pick different tones and
details during different times of the day when looking at the ocean.
This is important to understand because only then can we aim toward the
visual needs to accomplish a water shader with satisfactory fidelity.

![](refs/abyssal/assets/08bd032087bd39e622349cbc1cf24af6da755905.jpg)

To get that shimmering reflected sky light that we see over every little
waveling or surface distortion we need to add a *Fresnel* effect, as
this way the ocean shader is enriched with that ever-important highlight
to the shapes that represent the wavelings.

![](refs/abyssal/assets/7ad19dcb4fa82851dc1bd0023a91f986ed79d8d1.jpg)

Without Fresnel effect -- With Fresnel effect

As for the *color* of the water itself, it's always somewhere between
dark blue and dark green tints due to how light disperses on the surface
-- darker close to the camera and a little brighter farther from the
camera because of how the Fresnel effect works: light reflects best at a
lower angle of incidence on a water surface. And the more the surface
reflects light, the less we are able to see through it. That's why the
transmission factor will let us see into shallow water at a closer
distance with the obvious *Refraction* involved.

![](refs/abyssal/assets/6fba6291555a3ec5e543d7133db021252cfeea7a.png)

How light refracts below the surface is also the reason why objects in
shallow water can be seen as distorted.

For a final lighting detail we need to simulate the diffusion of direct
sunlight inside the wavelings of the water when facing the sun. By using
a *Subsurface Scattering* function we can simulate the result of a given
directional light by changing its direction and even choose the tint at
will.

![](refs/abyssal/assets/99cbca95b856291386c06f9ee3af1de7c5f1b5a9.jpg)

Without sub surface scattering -- With sub surface scattering

With these shading features done, we've got the desirable overall look
for the ocean shader but something is still missing -- interaction.

### What the foam is that?

Foam is important for interaction and at Abyssal our main goal was to
make sure that any object in contact with the ocean would generate it,
especially when currents are strong. To make that possible we used
*Distance Fields* data. How they work is no different than, let's say,
if every object had its own force field, and so we manage to generate
foam every time those fields overlap with our ocean surface. Being
directional oriented, we can take out only a directional portion of that
same overlap field, allowing us to generate foam only in the direction
of our ocean current. All the rest is texture work that complies to the
flow of the sea currents, distortions, and specular values. Distance
fields are also useful for controlling the vertex height when objects
such as rocks or floating objects are in contact with the ocean surface,
making it possible to get a field distortion in the water flow like it
is supposed to happen in a real scenario.

![](refs/abyssal/assets/761abda0bdc91197c99ed647015cfae1673a95d0.jpg)

Distance fields working like a force field for the objects that are in
contact with the surface of the ocean.

After the water shader is done, you still need to do a lot of extra
polishing and code work, especially if you want to dynamically change
its parameters in real-time for given sea or wind currents and any other
detail that might be influenced by weather and oceanic conditions. This
is where things get even more technical, as each project will have its
own requisites and specifics that demand different approaches to
controllers or parametric solutions. But let's skip that part and jump
into another important feature that definitely requires some more
logical hints and tricks: Capillary waves.

### Let's ripple things up

Capillary waves, also commonly known as ripples, are the immediate
reaction to the interaction with water. Unlike the contact distortion we
made using Distance Fields, these are not easily replicated because of
their fluid nature on a water surface. Ripples follow behind any object
that moves fast enough to leave a trail and diverge if they collide with
other ripples until their energy disperses completely. To accomplish
this, although too complex to be completely accurate, we need to
*capture* in real time all the objects near the camera that are in
contact with the ocean surface and turn that mapping into a world
position referenced texture that will be converted into specific texture
maps which are in turn injected into the water shader.

![](refs/abyssal/assets/29d3c10cdfc30351e08ba399eafa0c4f493363db.jpg)

The ROV leaves a trail on the water's surface as it moves.

Since this operation is constantly running and the engine needs to check
which objects are in contact with the surface every instant, this
quickly becomes a very heavy process. For this reason there's a need to
optimize some of the logic in a smart way that won't make running the
simulation smoothly a cumbersome problem.

### Conclusion

At the end of the day, the ocean is a small but very important component
of every project at Abyssal, with its potential not only essential but
pivotal. It's not about approaching realism but above all else doing it
while offering the tools and features needed to unequivocally enhance
our clients' experience and work performance. As a Technical Artist I
firmly believe that there's always room for improvement, pushing beyond
the horizon is but the starting point for a *fin-tastic* new challenge.

![](refs/abyssal/assets/4fc9942c269b1bd571e5de33e5a6bc23c7ff4970.png)
