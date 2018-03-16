# deltav
a rust command line tool to rapidly calculate delta v to change orbits

## A brief note on current limitations and future direction
Right now this tool will be only for in-plane orbits with a common apse line around a single central body. Additionally, there is no support currently for hyperbolic orbits just to keep things simple while I am getting things up and running with appropriate tests. Also assumes only homman transfer

### Future Work
- [ ] Hyperbolic orbits
- [ ] Apse line rotation
- [ ] Support for out of plane maneuvers
- [ ] Orbit graphing
- [ ] Option for advanced orbit propagation
- [ ] option for simple aerobraking estimate

## Equations

### R_p
\[
r_p = \frac{h^2}{\mu (1 + e}
\]

### e 
\[
e = \frac{r_a - r_p}{r_a + r_p}
\]

### Time, hommann
\[
T_h = \frac{2 \pi}{\sqrt{\mu}}a^{3/2}
\]
