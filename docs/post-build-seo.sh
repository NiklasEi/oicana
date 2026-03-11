#!/usr/bin/env bash
set -euo pipefail

DIST_DIR="${1:-book/dist}"
SITE_URL="${2:-https://docs.oicana.com}"

echo "Running post-build SEO enhancements on $DIST_DIR..."

# --- Generate robots.txt ---
cat > "$DIST_DIR/robots.txt" <<EOF
User-agent: *
Allow: /

Sitemap: ${SITE_URL}/sitemap.xml
EOF
echo "Created robots.txt"

# --- Generate sitemap.xml ---
{
  echo '<?xml version="1.0" encoding="UTF-8"?>'
  echo '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"'
  echo '        xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"'
  echo '        xsi:schemaLocation="http://www.sitemaps.org/schemas/sitemap/0.9'
  echo '        http://www.sitemaps.org/schemas/sitemap/0.9/sitemap.xsd">'

  find "$DIST_DIR" -name '*.html' -type f | sort | while read -r file; do
    rel_path="${file#"$DIST_DIR"/}"
    if [ "$rel_path" = "index.html" ]; then
      url_path="/"
    else
      url_path="/${rel_path}"
    fi
    echo "  <url>"
    echo "    <loc>${SITE_URL}${url_path}</loc>"
    echo "  </url>"
  done

  echo '</urlset>'
} > "$DIST_DIR/sitemap.xml"
echo "Created sitemap.xml"

# --- Inject SEO meta tags into HTML files ---
inject_meta_tags() {
  local file="$1"
  local rel_path="${file#"$DIST_DIR"/}"

  local canonical_url
  if [ "$rel_path" = "index.html" ]; then
    canonical_url="${SITE_URL}/"
  else
    canonical_url="${SITE_URL}/${rel_path}"
  fi

  local title
  title=$(grep -oP '(?<=<title>).*?(?=</title>)' "$file" | head -1)
  if [ -z "$title" ]; then
    title="Oicana Documentation"
  fi

  local description
  description=$(grep -oP '(?<=<meta name="description" content=")[^"]*' "$file" | head -1)
  if [ -z "$description" ]; then
    description="Oicana - seamless PDF templating across multiple platforms using Typst."
  fi
  local og_description="${description:0:160}"

  local meta_tags
  meta_tags=$(cat <<METATAGS
    <link rel="canonical" href="${canonical_url}">
    <meta property="og:title" content="${title}">
    <meta property="og:description" content="${og_description}">
    <meta property="og:url" content="${canonical_url}">
    <meta property="og:site_name" content="Oicana">
    <meta property="og:type" content="article">
    <meta property="og:locale" content="en_US">
    <meta name="twitter:card" content="summary">
    <meta name="twitter:title" content="${title}">
    <meta name="twitter:description" content="${og_description}">
METATAGS
)

  local tmp_file="${file}.tmp"
  awk -v tags="$meta_tags" '
    /<meta name="generator"/ {
      print
      print tags
      next
    }
    { print }
  ' "$file" > "$tmp_file"
  mv "$tmp_file" "$file"
}

export -f inject_meta_tags
export DIST_DIR SITE_URL

find "$DIST_DIR" -name '*.html' -type f | while read -r file; do
  inject_meta_tags "$file"
done
echo "Injected OG, Twitter, and canonical meta tags into all HTML files"

echo "SEO post-build complete!"
