﻿using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using System.Text;
using System.Text.RegularExpressions;
using System.Xml.Linq;

namespace GroupedNativeMethodsGenerator;

[Generator(LanguageNames.CSharp)]
public partial class GroupedNativeMethodsGenerator : IIncrementalGenerator
{
    public void Initialize(IncrementalGeneratorInitializationContext context)
    {
        context.RegisterPostInitializationOutput(ctx =>
        {
            ctx.AddSource("GroupedNativeMethodsGenerator.Attribute.cs", """
namespace GroupedNativeMethodsGenerator
{
    [AttributeUsage(AttributeTargets.Class, AllowMultiple = false, Inherited = false)]
    internal sealed class GroupedNativeMethodsAttribute : Attribute
    {
        public string RemovePrefix { get; }
        public string RemoveSuffix { get; }
        public bool RemoveUntilTypeName { get; }
        public bool FixMethodName { get; }

        public GroupedNativeMethodsAttribute(string removePrefix = "", string removeSuffix = "", bool removeUntilTypeName = true, bool fixMethodName = true)
        {
            this.RemovePrefix = removePrefix;
            this.RemoveSuffix = removeSuffix;
            this.RemoveUntilTypeName = removeUntilTypeName;
            this.FixMethodName = fixMethodName;
        }
    }
}
""");
        });

        var source = context.SyntaxProvider.ForAttributeWithMetadataName("GroupedNativeMethodsGenerator.GroupedNativeMethodsAttribute",
            (node, token) => node is ClassDeclarationSyntax,
            (ctx, token) => ctx);

        context.RegisterSourceOutput(source, Emit);
    }

    static void Emit(SourceProductionContext context, GeneratorAttributeSyntaxContext source)
    {
        var typeSymbol = (INamedTypeSymbol)source.TargetSymbol;
        var typeNode = (TypeDeclarationSyntax)source.TargetNode;

        var ns = typeSymbol.ContainingNamespace.IsGlobalNamespace
            ? ""
            : $"namespace {typeSymbol.ContainingNamespace}\n{{";

        var accessibility = typeSymbol.DeclaredAccessibility == Accessibility.Public ? "public" : "internal";

        var fullType = typeSymbol.ToDisplayString(SymbolDisplayFormat.FullyQualifiedFormat)
            .Replace("global::", "")
            .Replace("<", "_")
            .Replace(">", "_");

        var grouped = typeSymbol.GetMembers().OfType<IMethodSymbol>()
            .Where(x => x.Parameters.Length != 0)
            .Where(x => x.Parameters[0].Type is IPointerTypeSymbol t && (t.PointedAtType.SpecialType is SpecialType.None) && t.PointedAtType.TypeKind != TypeKind.Pointer)
            .ToLookup(x =>
            {
                return ((IPointerTypeSymbol)x.Parameters[0].Type).PointedAtType.ToDisplayString();
            });

        var libTypeName = typeSymbol.Name;
        var removePrefix = (string)source.Attributes[0].ConstructorArguments[0].Value!;
        var removeSuffix = (string)source.Attributes[0].ConstructorArguments[1].Value!;
        var removeUntilTypeName = (bool)source.Attributes[0].ConstructorArguments[2].Value!;
        var fixMethodName = (bool)source.Attributes[0].ConstructorArguments[3].Value!;

        var code = new StringBuilder();

        code.AppendLine($$"""
// <auto-generated/>
#nullable enable
#pragma warning disable CS8600
#pragma warning disable CS8601
#pragma warning disable CS8602
#pragma warning disable CS8603
#pragma warning disable CS8604

using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

{{ns}}

    {{accessibility}} static unsafe class {{typeSymbol.Name}}GroupingExtensions
    {
""");
        foreach (var g in grouped)
        {
            code.AppendLine($"#region {g.Key}({g.Count()})");
            code.AppendLine();
            foreach (var item in g)
            {
                var firstArgument = item.Parameters[0];
                var ret = item.ReturnType.ToDisplayString(SymbolDisplayFormat.FullyQualifiedFormat);
                var requireRet = ret == "void" ? "" : "return ";

                string? summaryComment = null;
                var docComment = item.GetDocumentationCommentXml();
                if (!string.IsNullOrEmpty(docComment))
                {
                    var xElem = XElement.Parse(docComment);
                    summaryComment = "/// " + xElem.Element("summary").ToString();
                }

                var convertedMethodName = ConvertMethodName(((IPointerTypeSymbol)firstArgument.Type).PointedAtType.Name, item.Name, removePrefix, removeSuffix, removeUntilTypeName, fixMethodName);
                var pointedType = ((IPointerTypeSymbol)firstArgument.Type).PointedAtType.ToDisplayString(SymbolDisplayFormat.FullyQualifiedFormat);
                var parameterPairs = string.Join("", item.Parameters.Skip(1).Select(x => $", {x.Type.ToDisplayString(SymbolDisplayFormat.FullyQualifiedFormat)} @{x.Name}"));
                var parameterNames = string.Join("", item.Parameters.Skip(1).Select(x => $", @{x.Name}"));

                if (summaryComment != null)
                {
                    code.AppendLine("        " + summaryComment);
                }
                code.AppendLine($"        public static {ret} {convertedMethodName}(this ref {pointedType} @{firstArgument.Name}{parameterPairs})");
                code.AppendLine("        {");
                code.AppendLine($"            {requireRet}{libTypeName}.{item.Name}(({pointedType}*)Unsafe.AsPointer(ref @{firstArgument.Name}){parameterNames});");
                code.AppendLine("        }");
                code.AppendLine("");
            }
            code.AppendLine($"#endregion");
            code.AppendLine();
        }

        code.AppendLine("    }");
        if (ns != "")
        {
            code.AppendLine("}");
        }

        context.AddSource($"{fullType}.GroupedNativeMethods.g.cs", code.ToString());
    }

    static string ConvertMethodName(string typeName, string methodName, string removePrefix, string removeSuffix, bool removeUntilTypeName, bool fixMethodName)
    {
        if (!fixMethodName) return methodName;

        if (removeUntilTypeName)
        {
            var match = methodName.IndexOf(typeName);
            if (match != -1)
            {
                var substringMethodName = methodName.Substring(match + typeName.Length);
                if (substringMethodName.Trim(' ', '_') != "")
                {
                    methodName = substringMethodName;
                    goto FINAL;
                }
            }
        }

        if (!string.IsNullOrEmpty(removePrefix))
        {
            methodName = Regex.Replace(methodName, $"^{Regex.Escape(removePrefix)}", "");
        }

    FINAL:

        if (!string.IsNullOrEmpty(removeSuffix))
        {
            methodName = Regex.Replace(methodName, $"{Regex.Escape(removeSuffix)}$", "");
        }

        methodName = methodName.Trim('_', ' ');

        var split = methodName.Split('_');
        methodName = string.Concat(split.Select(x =>
        {
            if (x.Length == 0) return x;
            if (x.Length == 1) return char.ToUpper(x[0]).ToString();
            return char.ToUpper(x[0]) + x.Substring(1);
        }));

        return methodName;
    }
}