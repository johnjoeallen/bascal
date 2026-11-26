import java.lang.classfile.*;
import java.lang.classfile.instruction.*;
import java.lang.constant.*;
import java.lang.reflect.AccessFlag;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;

/**
 * Assembles the bcc JVM backend's text listing (".j") into a class file with
 * the JDK ClassFile API. Run as: java Assemble.java <in.j> <out.class> [<outdir>]
 *
 * Stack map frames and max_stack/max_locals are computed by the API, so the
 * listing has no .version/.limit/.stack directives (they are rejected).
 * Errors are reported as "<file>:<line>: <message>" with exit status 1.
 * The class name is taken from the .class directive; the class file is
 * written to <out.class>.
 */
public final class Assemble {
    static final class AsmError extends RuntimeException {
        final int line;
        AsmError(int line, String msg) { super(msg); this.line = line; }
    }

    record Src(int no, String text) {}
    record Catch(int no, String type, String from, String to, String handler) {}

    static final class Member {
        String kind; // field | method
        int no;
        int flags;
        String name, desc;
        List<Src> body = new ArrayList<>();
        List<Catch> catches = new ArrayList<>();
    }

    public static void main(String[] args) throws Exception {
        if (args.length != 2) {
            System.err.println("usage: java Assemble.java <in.j> <out.class>");
            System.exit(2);
        }
        String file = args[0];
        try {
            byte[] bytes = assemble(Files.readAllLines(Path.of(file), StandardCharsets.UTF_8));
            Files.write(Path.of(args[1]), bytes);
        } catch (AsmError e) {
            System.err.println(file + ":" + e.line + ": " + e.getMessage());
            System.exit(1);
        } catch (IllegalArgumentException | IllegalStateException | ClassFormatError e) {
            System.err.println(file + ": class generation failed: " + e.getMessage());
            System.exit(1);
        }
    }

    static String stripComment(String s) {
        boolean inStr = false;
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            if (inStr) {
                if (c == '\\') i++;
                else if (c == '"') inStr = false;
            } else if (c == '"') inStr = true;
            else if (c == ';' && (i == 0 || Character.isWhitespace(s.charAt(i - 1)))) return s.substring(0, i);
        }
        return s;
    }

    static int flags(int no, String[] toks, int from, int to, boolean method) {
        int f = 0;
        for (int i = from; i < to; i++) {
            switch (toks[i]) {
                case "public" -> f |= ClassFile.ACC_PUBLIC;
                case "private" -> f |= ClassFile.ACC_PRIVATE;
                case "protected" -> f |= ClassFile.ACC_PROTECTED;
                case "static" -> f |= ClassFile.ACC_STATIC;
                case "final" -> f |= ClassFile.ACC_FINAL;
                default -> throw new AsmError(no, "unknown access flag '" + toks[i] + "'");
            }
        }
        return f;
    }

    static ClassDesc classDesc(int no, String name) {
        try {
            return name.startsWith("[") ? ClassDesc.ofDescriptor(name) : ClassDesc.ofInternalName(name);
        } catch (IllegalArgumentException e) {
            throw new AsmError(no, "bad class name '" + name + "'");
        }
    }

    static byte[] assemble(List<String> lines) {
        String className = null, superName = "java/lang/Object";
        List<Member> members = new ArrayList<>();
        Member cur = null;
        for (int i = 0; i < lines.size(); i++) {
            int no = i + 1;
            String t = stripComment(lines.get(i)).strip();
            if (t.isEmpty()) continue;
            if (t.startsWith(".")) {
                String[] tk = t.split("\\s+");
                switch (tk[0]) {
                    case ".class" -> className = tk[tk.length - 1];
                    case ".super" -> superName = tk[1];
                    case ".field" -> {
                        if (tk.length < 3) throw new AsmError(no, "malformed .field");
                        Member m = new Member();
                        m.kind = "field"; m.no = no;
                        m.flags = flags(no, tk, 1, tk.length - 2, false);
                        m.name = tk[tk.length - 2]; m.desc = tk[tk.length - 1];
                        members.add(m);
                    }
                    case ".method" -> {
                        int colon = Arrays.asList(tk).indexOf(":");
                        if (colon < 2 || colon + 1 >= tk.length) throw new AsmError(no, "malformed .method");
                        cur = new Member();
                        cur.kind = "method"; cur.no = no;
                        cur.flags = flags(no, tk, 1, colon - 1, true);
                        cur.name = tk[colon - 1]; cur.desc = tk[colon + 1];
                        members.add(cur);
                    }
                    case ".end" -> {
                        if (tk.length != 2 || !tk[1].equals("method")) throw new AsmError(no, "malformed .end");
                        cur = null;
                    }
                    case ".catch" -> {
                        // .catch <type> from <L> to <L> using <L>
                        if (cur == null || tk.length != 8 || !tk[2].equals("from") || !tk[4].equals("to") || !tk[6].equals("using"))
                            throw new AsmError(no, "malformed .catch");
                        cur.catches.add(new Catch(no, tk[1], tk[3], tk[5], tk[7]));
                    }
                    default -> throw new AsmError(no, "unknown directive " + tk[0]);
                }
                continue;
            }
            if (cur == null) throw new AsmError(no, "instruction outside a method: " + t);
            cur.body.add(new Src(no, t));
        }
        if (className == null) throw new AsmError(1, "missing .class directive");
        final String cn = className, sn = superName;
        ClassDesc self = ClassDesc.ofInternalName(cn);
        return ClassFile.of().build(self, cb -> {
            cb.withVersion(69, 0);
            cb.withFlags(ClassFile.ACC_PUBLIC | ClassFile.ACC_SUPER);
            cb.withSuperclass(ClassDesc.ofInternalName(sn));
            for (Member m : members) {
                if (m.kind.equals("field")) {
                    cb.withField(m.name, ClassDesc.ofDescriptor(m.desc), m.flags);
                } else {
                    MethodTypeDesc mt;
                    try { mt = MethodTypeDesc.ofDescriptor(m.desc); }
                    catch (IllegalArgumentException e) { throw new AsmError(m.no, "bad method descriptor " + m.desc); }
                    cb.withMethod(m.name, mt, m.flags, mb -> mb.withCode(code -> emit(code, m)));
                }
            }
        });
    }

    static String unquote(int no, String s) {
        if (s.length() < 2 || s.charAt(0) != '"' || s.charAt(s.length() - 1) != '"')
            throw new AsmError(no, "bad string constant " + s);
        StringBuilder sb = new StringBuilder();
        for (int i = 1; i < s.length() - 1; i++) {
            char c = s.charAt(i);
            if (c != '\\') { sb.append(c); continue; }
            char n = s.charAt(++i);
            switch (n) {
                case 'n' -> sb.append('\n');
                case 't' -> sb.append('\t');
                case 'r' -> sb.append('\r');
                case '"' -> sb.append('"');
                case '\\' -> sb.append('\\');
                case 'u' -> { sb.append((char) Integer.parseInt(s.substring(i + 1, i + 5), 16)); i += 4; }
                default -> throw new AsmError(no, "bad escape \\" + n);
            }
        }
        return sb.toString();
    }

    static int intArg(int no, String s) {
        try { return Integer.parseInt(s); }
        catch (NumberFormatException e) { throw new AsmError(no, "bad integer " + s); }
    }

    static TypeKind loadKind(Opcode op) {
        return switch (op) {
            case ILOAD, ISTORE -> TypeKind.INT;
            case LLOAD, LSTORE -> TypeKind.LONG;
            case FLOAD, FSTORE -> TypeKind.FLOAT;
            case DLOAD, DSTORE -> TypeKind.DOUBLE;
            default -> TypeKind.REFERENCE;
        };
    }

    static void emit(CodeBuilder b, Member m) {
        Map<String, Label> labels = new HashMap<>();
        java.util.function.Function<String, Label> lab = n -> labels.computeIfAbsent(n, k -> b.newLabel());
        List<Src> body = m.body;
        for (int i = 0; i < body.size(); i++) {
            Src s = body.get(i);
            int no = s.no();
            String t = s.text();
            if (t.endsWith(":") && !t.contains(" ")) {
                b.labelBinding(lab.apply(t.substring(0, t.length() - 1)));
                continue;
            }
            int sp = t.indexOf(' ');
            String mn = sp < 0 ? t : t.substring(0, sp);
            String rest = sp < 0 ? "" : t.substring(sp + 1).strip();
            String[] a = rest.isEmpty() ? new String[0] : rest.split("\\s+");
            Opcode op;
            try { op = Opcode.valueOf(mn.toUpperCase(Locale.ROOT)); }
            catch (IllegalArgumentException e) { throw new AsmError(no, "unknown instruction '" + mn + "'"); }
            switch (op.kind()) {
                case LOAD -> b.loadLocal(loadKind(op), intArg(no, a[0]));
                case STORE -> b.storeLocal(loadKind(op), intArg(no, a[0]));
                case INCREMENT -> b.iinc(intArg(no, a[0]), intArg(no, a[1]));
                case BRANCH -> b.branch(op, lab.apply(a[0]));
                case TABLE_SWITCH -> {
                    int low = intArg(no, a[0]);
                    List<SwitchCase> cases = new ArrayList<>();
                    Label dflt = null;
                    int v = low;
                    while (dflt == null) {
                        Src n = body.get(++i);
                        String[] w = n.text().split("\\s+");
                        if (w[0].equals("default")) dflt = lab.apply(w[w.length - 1]);
                        else cases.add(SwitchCase.of(v++, lab.apply(w[0])));
                    }
                    b.tableswitch(low, v - 1, dflt, cases);
                }
                case FIELD_ACCESS -> {
                    int slash = a[0].lastIndexOf('/');
                    b.fieldAccess(op, classDesc(no, a[0].substring(0, slash)), a[0].substring(slash + 1), ClassDesc.ofDescriptor(a[1]));
                }
                case INVOKE -> {
                    int slash = a[0].lastIndexOf('/');
                    String desc = a[1];
                    b.invoke(op, classDesc(no, a[0].substring(0, slash)), a[0].substring(slash + 1),
                            MethodTypeDesc.ofDescriptor(desc), op == Opcode.INVOKEINTERFACE);
                }
                case NEW_OBJECT -> b.new_(classDesc(no, a[0]));
                case NEW_REF_ARRAY -> b.anewarray(classDesc(no, a[0]));
                case NEW_MULTI_ARRAY -> b.multianewarray(classDesc(no, a[0]), intArg(no, a[1]));
                case NEW_PRIMITIVE_ARRAY -> {
                    TypeKind k = switch (a[0]) {
                        case "boolean" -> TypeKind.BOOLEAN; case "char" -> TypeKind.CHAR;
                        case "float" -> TypeKind.FLOAT; case "double" -> TypeKind.DOUBLE;
                        case "byte" -> TypeKind.BYTE; case "short" -> TypeKind.SHORT;
                        case "int" -> TypeKind.INT; case "long" -> TypeKind.LONG;
                        default -> throw new AsmError(no, "bad newarray type " + a[0]);
                    };
                    b.newarray(k);
                }
                case TYPE_CHECK -> {
                    if (op == Opcode.CHECKCAST) b.checkcast(classDesc(no, a[0]));
                    else b.instanceOf(classDesc(no, a[0]));
                }
                case CONSTANT -> {
                    switch (op) {
                        case LDC, LDC_W -> {
                            if (rest.startsWith("\"")) b.ldc(b.constantPool().stringEntry(unquote(no, rest)));
                            else if (rest.endsWith("f")) b.loadConstant(Float.parseFloat(rest));
                            else b.loadConstant(intArg(no, rest));
                        }
                        case LDC2_W -> {
                            if (rest.endsWith("L")) b.loadConstant(Long.parseLong(rest.substring(0, rest.length() - 1)));
                            else b.loadConstant(Double.parseDouble(rest));
                        }
                        case BIPUSH, SIPUSH -> b.loadConstant(intArg(no, rest));
                        default -> b.with(ConstantInstruction.ofIntrinsic(op));
                    }
                }
                case NOP -> b.nop();
                case RETURN -> b.with(ReturnInstruction.of(op));
                case ARRAY_LOAD -> b.with(ArrayLoadInstruction.of(op));
                case ARRAY_STORE -> b.with(ArrayStoreInstruction.of(op));
                case STACK -> b.with(StackInstruction.of(op));
                case CONVERT -> b.with(ConvertInstruction.of(op));
                case OPERATOR -> b.with(OperatorInstruction.of(op));
                case THROW_EXCEPTION -> b.athrow();
                case MONITOR -> b.with(MonitorInstruction.of(op));
                default -> throw new AsmError(no, "unsupported instruction '" + mn + "'");
            }
        }
        for (Catch c : m.catches) {
            b.exceptionCatch(lab.apply(c.from()), lab.apply(c.to()), lab.apply(c.handler()), classDesc(c.no(), c.type()));
        }
    }
}
