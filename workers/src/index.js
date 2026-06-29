const GITHUB_RAW = "https://raw.githubusercontent.com/suraniharsh/kairos/main";
const GITHUB_REPO = "https://github.com/suraniharsh/kairos";

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

    // Root → GitHub repo
    return Response.redirect(GITHUB_REPO, 302);
  },
};
