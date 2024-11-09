helm repo add cnpg https://cloudnative-pg.github.io/charts
helm upgrade --install cnpg \
  --namespace postgres \
  --create-namespace \
  cnpg/cloudnative-pg