# Deployment

Once the docker image is built, the project can be deployed on any kubernetes cluster, either by plain manifests or by a helm chart.

As the project is simple enough, it can be deployed using a generic application chart like [Stakater Application](https://github.com/stakater/application) helm chart.
Example values files required to deploy the project with this helm chart can be found under `./contrib/helm` folder. 

In particular, these are the basic values you will need to change:

- Docker image name and tag
- Ingress url
- Specific configurations on the `settings.toml` must be set to match your own infrastructure
  - tendermint addr and port
  - db connection params
  