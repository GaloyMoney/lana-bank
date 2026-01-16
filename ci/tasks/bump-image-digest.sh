#!/bin/bash
set -eu

export digest=$(cat ./latest-image/digest)
export admin_panel_image_digest=$(cat ./admin-panel-latest-image/digest)
export customer_portal_image_digest=$(cat ./customer-portal-latest-image/digest)
export dagster_image_digest=$(cat ./dagster-image/digest)

export ref=$(cat ./repo/.git/short_ref)
export app_version=$(cat version/version)

pushd charts-repo

yq -i e '.lanaBank.image.digest = strenv(digest)' ./charts/${CHARTS_SUBDIR}/values.yaml
yq -i e '.lanaBank.adminPanel.image.digest = strenv(admin_panel_image_digest)' ./charts/${CHARTS_SUBDIR}/values.yaml
yq -i e '.lanaBank.customerPortal.image.digest = strenv(customer_portal_image_digest)' ./charts/${CHARTS_SUBDIR}/values.yaml
yq -i e '.dagster.dagster-user-deployments.deployments[0].image.digest = strenv(dagster_image_digest)' ./charts/${CHARTS_SUBDIR}/values.yaml
# Temporarily disabled because there's bug in the private chart and Pablo can't manage it right now, so stop bumping for a bit
#yq -i e '.dagster.dagster-user-deployments.deployments[0].initContainers[0].image.digest = strenv(dagster_image_digest)' ./charts/${CHARTS_SUBDIR}/values.yaml
sed -i "s|\(digest: \"${digest}\"\).*$|\1 # METADATA:: repository=https://github.com/GaloyMoney/${CHARTS_SUBDIR};commit_ref=${ref};app=${CHARTS_SUBDIR};|g" "./charts/${CHARTS_SUBDIR}/values.yaml"

yq -i e '.lanaBank.appVersion = strenv(app_version)' ./charts/${CHARTS_SUBDIR}/values.yaml
yq -i e '.appVersion = strenv(app_version)' ./charts/${CHARTS_SUBDIR}/Chart.yaml

rm -rf ./charts/${CHARTS_SUBDIR}/tf || true
mkdir -p ./charts/${CHARTS_SUBDIR}/tf
cp -r ../repo/tf/bq-setup ./charts/${CHARTS_SUBDIR}/tf/bq-setup
cp -r ../repo/tf/honeycomb ./charts/${CHARTS_SUBDIR}/tf/honeycomb
cat ../repo/.git/ref > ./charts/${CHARTS_SUBDIR}/tf/repo-ref

if [[ -z $(git config --global user.email) ]]; then
  git config --global user.email "bot@galoy.io"
fi
if [[ -z $(git config --global user.name) ]]; then
  git config --global user.name "CI Bot"
fi

(
  cd $(git rev-parse --show-toplevel)
  git merge --no-edit ${BRANCH}
  git add -A
  git status
  git commit -m "chore(${CHARTS_SUBDIR}): bump ${CHARTS_SUBDIR} image to '${digest}'"
)

