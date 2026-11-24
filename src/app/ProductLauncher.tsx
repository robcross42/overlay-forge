import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

interface Product { key: string; name: string; available: boolean }
export function ProductLauncher({ sessionId }: { sessionId: string }) {
  const [products, setProducts] = useState<Product[]>([]);
  const [status, setStatus] = useState("");
  async function refresh() {
    try { setProducts(await invoke<Product[]>("list_products")); }
    catch (error) { setStatus(String(error)); }
  }
  useEffect(() => { void refresh(); }, []);
  return <section className="panel">
    <div className="row"><h2>Your products</h2><button onClick={() => void refresh()}>Refresh</button></div>
    <p role="status">{status}</p>
    <div className="products">{products.map(product => <button key={product.key} disabled={!product.available} onClick={() => {
      setStatus(`Launching ${product.name}…`);
      void invoke("launch_product", { productKey: product.key, sessionId })
        .then(() => setStatus(`${product.name} opened. Overlay Forge remains resident.`))
        .catch(error => setStatus(String(error)));
    }}><strong>{product.name}</strong><small>{product.available ? "Open product" : product.key === "journal" ? "Start its local development server" : "Build or install this product first"}</small></button>)}</div>
  </section>;
}
