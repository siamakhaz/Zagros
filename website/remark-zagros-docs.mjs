// The page template already supplies the documentation H1.
export function remarkZagrosDocs() {
  return (tree) => {
    function visit(node) {
      if (node.type === 'heading' && node.depth === 1) node.depth = 2;
      if (Array.isArray(node.children)) node.children.forEach(visit);
    }
    visit(tree);
  };
}
