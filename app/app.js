const shell = document.querySelector(".shell");
const toggle = document.querySelector(".sidebar-toggle");

const savedSidebarState = localStorage.getItem("arkan.sidebar");

if (savedSidebarState === "collapsed") {
  shell.dataset.sidebar = "collapsed";
  toggle.setAttribute("aria-label", "Expand sidebar");
}

toggle.addEventListener("click", () => {
  const collapsed = shell.dataset.sidebar === "collapsed";
  shell.dataset.sidebar = collapsed ? "expanded" : "collapsed";
  localStorage.setItem("arkan.sidebar", collapsed ? "expanded" : "collapsed");
  toggle.setAttribute("aria-label", collapsed ? "Reduce sidebar" : "Expand sidebar");
});
