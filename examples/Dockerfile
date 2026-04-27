FROM nginx:1.27-alpine
COPY nginx-static.conf /etc/nginx/conf.d/default.conf
# Replace relay-pin-manifest.example.json with your signed output from `ombifest sign` or `ombifest build-relay`.
COPY relay-pin-manifest.example.json /usr/share/nginx/html/relay-pin-manifest.json
