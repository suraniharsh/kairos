const GITHUB_RAW = "https://raw.githubusercontent.com/suraniharsh/kairos/main";
const GITHUB_REPO = "https://github.com/suraniharsh/kairos";
const GITHUB_LATEST_RELEASE_API =
  "https://api.github.com/repos/suraniharsh/kairos/releases/latest";

export default {
  async fetch(request) {
    const url = new URL(request.url);
    const path = url.pathname;

    if (path === "/install.sh" || path === "/install.ps1") {
      const res = await fetch(`${GITHUB_RAW}${path}`);
      return new Response(res.body, {
        status: res.status,
        headers: {
          "Content-Type": "text/plain; charset=utf-8",
          "Cache-Control": "no-cache",
        },
      });
    }

    if (path === "/version") {
      // Proxied (rather than hit by every client directly) so the update
      // check works from a stable, unauthenticated endpoint that isn't
      // subject to GitHub's per-IP anonymous rate limit.
      const res = await fetch(GITHUB_LATEST_RELEASE_API, {
        headers: {
          Accept: "application/vnd.github+json",
          "User-Agent": "kairos-worker",
        },
      });
      if (!res.ok) {
        return new Response(JSON.stringify({ error: "upstream error" }), {
          status: 502,
          headers: { "Content-Type": "application/json" },
        });
      }
      const release = await res.json();
      return new Response(JSON.stringify({ version: release.tag_name }), {
        headers: {
          "Content-Type": "application/json",
          // Short edge cache so a burst of client checks collapses to one
          // GitHub API call, while still surfacing a new release quickly.
          "Cache-Control": "public, max-age=300",
        },
      });
    }

    // Root → GitHub repo
    return Response.redirect(GITHUB_REPO, 302);
  },
};
